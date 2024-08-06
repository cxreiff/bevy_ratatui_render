// For more detailed comments on this general approach, please reference @bugsweeper's
// excellent headless_rendering bevy example:
//
// https://github.com/bevyengine/bevy/blob/main/examples/app/headless_renderer.rs

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_asset::{RenderAssetUsages, RenderAssets},
        render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel},
        render_resource::{
            Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d,
            ImageCopyBuffer, ImageDataLayout, Maintain, MapMode, TextureDimension, TextureFormat,
            TextureUsages,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::{BevyDefault, GpuImage, TextureFormatPixelInfo},
        Extract,
    },
};
use crossbeam_channel::{Receiver, Sender};

use crate::RatatuiRenderContext;

#[derive(Clone, Default, Resource, Deref, DerefMut)]
pub struct ImageCopiers(pub Vec<ImageCopier>);

#[derive(Clone, Component)]
pub struct ImageCopier {
    sender: Sender<Vec<u8>>,
    buffer: Buffer,
    enabled: Arc<AtomicBool>,
    pub src_image: Handle<Image>,
}

impl ImageCopier {
    pub fn new(
        src_image: Handle<Image>,
        (width, height): (u32, u32),
        render_device: &RenderDevice,
        sender: Sender<Vec<u8>>,
    ) -> ImageCopier {
        let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row((width) as usize) * 4;

        let cpu_buffer = render_device.create_buffer(&BufferDescriptor {
            label: None,
            size: padded_bytes_per_row as u64 * height as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        ImageCopier {
            buffer: cpu_buffer,
            src_image,
            enabled: Arc::new(AtomicBool::new(true)),
            sender,
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
}

pub struct HeadlessRenderPipe {
    receiver: Receiver<Vec<u8>>,
    pub target: RenderTarget,
    pub image: Image,
}

impl HeadlessRenderPipe {
    pub fn new(
        commands: &mut Commands,
        images: &mut Assets<Image>,
        render_device: &RenderDevice,
        dimensions: (u32, u32),
    ) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();

        let (render_texture, cpu_texture) = create_render_textures(dimensions);

        let render_handle = images.add(render_texture);

        commands.spawn(ImageCopier::new(
            render_handle.clone(),
            dimensions,
            render_device,
            sender,
        ));

        Self {
            receiver,
            target: RenderTarget::Image(render_handle),
            image: cpu_texture,
        }
    }
}

pub fn image_copier_extract_system(
    mut commands: Commands,
    image_copy_sources: Extract<Query<&ImageCopier>>,
) {
    commands.insert_resource(ImageCopiers(
        image_copy_sources
            .iter()
            .cloned()
            .collect::<Vec<ImageCopier>>(),
    ));
}

pub fn send_rendered_image_system(
    image_copy_sources: Res<ImageCopiers>,
    render_device: Res<RenderDevice>,
) {
    for image_copy_source in image_copy_sources.iter() {
        if !image_copy_source.enabled() {
            continue;
        }

        let buffer_slice = image_copy_source.buffer.slice(..);

        let (s, r) = crossbeam_channel::bounded(1);

        buffer_slice.map_async(MapMode::Read, move |r| match r {
            Ok(r) => s.send(r).expect("Failed to send map update"),
            Err(err) => panic!("Failed to map buffer {err}"),
        });

        render_device.poll(Maintain::wait()).panic_on_timeout();

        r.recv().expect("Failed to receive the map_async message");

        let _ = image_copy_source
            .sender
            .send(buffer_slice.get_mapped_range().to_vec());

        image_copy_source.buffer.unmap();
    }
}

pub fn receive_rendered_images_system(mut ratatui_render: ResMut<RatatuiRenderContext>) {
    for render_pipe in &mut ratatui_render.values_mut() {
        let mut image_data = Vec::new();
        while let Ok(data) = render_pipe.receiver.try_recv() {
            image_data = data;
        }
        if !image_data.is_empty() {
            let row_bytes = render_pipe.image.width() as usize
                * render_pipe.image.texture_descriptor.format.pixel_size();
            let aligned_row_bytes = RenderDevice::align_copy_bytes_per_row(row_bytes);
            if row_bytes == aligned_row_bytes {
                render_pipe.image.data.clone_from(&image_data);
            } else {
                render_pipe.image.data = image_data
                    .chunks(aligned_row_bytes)
                    .take(render_pipe.image.height() as usize)
                    .flat_map(|row| &row[..row_bytes.min(row.len())])
                    .cloned()
                    .collect();
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, RenderLabel)]
pub struct ImageCopy;

#[derive(Default)]
pub struct ImageCopyNode;

impl Node for ImageCopyNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let image_copy_sources = world.get_resource::<ImageCopiers>().unwrap();
        let gpu_images = world.get_resource::<RenderAssets<GpuImage>>().unwrap();

        for image_copy_source in image_copy_sources.iter() {
            if !image_copy_source.enabled() {
                continue;
            }

            let src_image = gpu_images.get(&image_copy_source.src_image).unwrap();

            let mut encoder = render_context
                .render_device()
                .create_command_encoder(&CommandEncoderDescriptor::default());

            let block_dimensions = src_image.texture_format.block_dimensions();
            let block_size = src_image.texture_format.block_copy_size(None).unwrap();

            let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(
                (src_image.size.x as usize / block_dimensions.0 as usize) * block_size as usize,
            );

            let texture_extent = Extent3d {
                width: src_image.size.x,
                height: src_image.size.y,
                depth_or_array_layers: 1,
            };

            encoder.copy_texture_to_buffer(
                src_image.texture.as_image_copy(),
                ImageCopyBuffer {
                    buffer: &image_copy_source.buffer,
                    layout: ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(
                            std::num::NonZeroU32::new(padded_bytes_per_row as u32)
                                .unwrap()
                                .into(),
                        ),
                        rows_per_image: None,
                    },
                },
                texture_extent,
            );

            let render_queue = world.get_resource::<RenderQueue>().unwrap();
            render_queue.submit(std::iter::once(encoder.finish()));
        }

        Ok(())
    }
}

fn create_render_textures(dimensions: (u32, u32)) -> (Image, Image) {
    let (width, height) = dimensions;
    let size = Extent3d {
        width,
        height,
        ..Default::default()
    };

    let mut render_texture = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0; 4],
        TextureFormat::bevy_default(),
        RenderAssetUsages::default(),
    );

    let cpu_texture = render_texture.clone();

    render_texture.texture_descriptor.usage |=
        TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;

    (render_texture, cpu_texture)
}
