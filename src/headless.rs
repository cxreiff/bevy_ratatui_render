use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use bevy::{
    image::TextureFormatPixelInfo,
    prelude::*,
    render::{
        camera::RenderTarget,
        extract_component::ExtractComponent,
        render_asset::{RenderAssetUsages, RenderAssets},
        render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel, ViewNode},
        render_resource::{
            Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d,
            ImageCopyBuffer, ImageDataLayout, Maintain, MapMode, TextureDimension, TextureFormat,
            TextureUsages,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::GpuImage,
    },
};
use crossbeam_channel::{Receiver, Sender};

use crate::{sobel::ImageCopierSobel, RatatuiRenderContext, RatatuiRenderStrategy};

#[derive(Component, ExtractComponent, Clone)]
pub struct ImageCopier {
    pub image: Handle<Image>,
    buffer: Buffer,
    sender: Sender<Vec<u8>>,
    enabled: Arc<AtomicBool>,
}

impl ImageCopier {
    pub fn new(image: Handle<Image>, buffer: Buffer, sender: Sender<Vec<u8>>) -> Self {
        Self {
            image,
            buffer,
            sender,
            enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
}

pub struct HeadlessRenderPipe {
    receiver: Receiver<Vec<u8>>,
    receiver_sobel: Option<Receiver<Vec<u8>>>,
    pub target: RenderTarget,
    pub image: Image,
    pub image_sobel: Option<Image>,
    pub strategy: RatatuiRenderStrategy,
}

impl HeadlessRenderPipe {
    pub fn new(
        commands: &mut Commands,
        images: &mut Assets<Image>,
        render_device: &RenderDevice,
        dimensions: (u32, u32),
        strategy: RatatuiRenderStrategy,
    ) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let (render_texture, cpu_texture) = create_render_textures(dimensions);
        let buffer = create_image_copier_buffer(render_device, dimensions);
        let render_handle = images.add(render_texture);

        let image_copier = ImageCopier::new(render_handle.clone(), buffer, sender);

        let mut image_copier_sobel = None;
        let mut cpu_texture_sobel = None;
        let mut receiver_sobel = None;
        if let RatatuiRenderStrategy::Luminance(ref luminance_config) = strategy {
            if luminance_config.edge_detection {
                let (sender, receiver) = crossbeam_channel::unbounded();
                let (render_texture, cpu_texture) = create_render_textures(dimensions);
                let render_handle = images.add(render_texture);
                let buffer = create_image_copier_buffer(render_device, dimensions);

                image_copier_sobel = Some(ImageCopierSobel::new(render_handle, buffer, sender));

                receiver_sobel = Some(receiver);
                cpu_texture_sobel = Some(cpu_texture);
            }
        };

        if let Some(image_copier_sobel) = image_copier_sobel {
            commands.spawn((image_copier, image_copier_sobel));
        } else {
            commands.spawn(image_copier);
        }

        Self {
            receiver,
            receiver_sobel,
            target: RenderTarget::Image(render_handle),
            image: cpu_texture,
            image_sobel: cpu_texture_sobel,
            strategy,
        }
    }
}

// pub fn image_copier_extract_system(
//     mut commands: Commands,
//     image_copy_sources: Extract<Query<&ImageCopier>>,
// ) {
//     commands.insert_resource(ImageCopiers(
//         image_copy_sources
//             .iter()
//             .cloned()
//             .collect::<Vec<ImageCopier>>(),
//     ));
// }

pub fn send_rendered_image_system(
    image_copy_sources: Query<(&ImageCopier, Option<&ImageCopierSobel>)>,
    render_device: Res<RenderDevice>,
) {
    for (image_copier, image_copier_sobel) in image_copy_sources.iter() {
        if !image_copier.enabled() {
            continue;
        }

        let buffer_slice = image_copier.buffer.slice(..);

        let (s, r) = crossbeam_channel::bounded(1);

        buffer_slice.map_async(MapMode::Read, move |r| match r {
            Ok(r) => s.send(r).expect("Failed to send map update"),
            Err(err) => panic!("Failed to map buffer {err}"),
        });

        render_device.poll(Maintain::wait()).panic_on_timeout();

        r.recv().expect("Failed to receive the map_async message");

        let _ = image_copier
            .sender
            .send(buffer_slice.get_mapped_range().to_vec());

        image_copier.buffer.unmap();
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

pub fn receive_sobel_images_system(mut ratatui_render: ResMut<RatatuiRenderContext>) {
    for render_pipe in &mut ratatui_render.values_mut() {
        let Some(ref receiver) = render_pipe.receiver_sobel else {
            return;
        };

        let Some(ref image) = render_pipe.image_sobel else {
            return;
        };

        let mut image_data = Vec::new();
        while let Ok(data) = receiver.try_recv() {
            image_data = data;
        }
        if !image_data.is_empty() {
            let row_bytes =
                image.width() as usize * render_pipe.image.texture_descriptor.format.pixel_size();
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

impl ViewNode for ImageCopyNode {
    type ViewQuery = &'static ImageCopier;

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        image_copier: bevy::ecs::query::QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let gpu_images = world.get_resource::<RenderAssets<GpuImage>>().unwrap();

        if !image_copier.enabled() {
            return Ok(());
        }

        let src_image = gpu_images.get(&image_copier.image).unwrap();

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
                buffer: &image_copier.buffer,
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

        Ok(())
    }
    // fn run(
    //     &self,
    //     _graph: &mut RenderGraphContext,
    //     render_context: &mut RenderContext,
    //     world: &World,
    // ) -> Result<(), NodeRunError> {
    //     // let image_copy_sources = world.get_resource::<ImageCopiers>().unwrap();
    //     let mut image_copiers = world.query::<&mut ImageCopier>();
    //     let gpu_images = world.get_resource::<RenderAssets<GpuImage>>().unwrap();

    //     for image_copy_source in image_copiers.iter(world) {
    //         if !image_copy_source.enabled() {
    //             continue;
    //         }

    //         let src_image = gpu_images.get(&image_copy_source.image).unwrap();

    //         let mut encoder = render_context
    //             .render_device()
    //             .create_command_encoder(&CommandEncoderDescriptor::default());

    //         let block_dimensions = src_image.texture_format.block_dimensions();
    //         let block_size = src_image.texture_format.block_copy_size(None).unwrap();

    //         let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(
    //             (src_image.size.x as usize / block_dimensions.0 as usize) * block_size as usize,
    //         );

    //         let texture_extent = Extent3d {
    //             width: src_image.size.x,
    //             height: src_image.size.y,
    //             depth_or_array_layers: 1,
    //         };

    //         encoder.copy_texture_to_buffer(
    //             src_image.texture.as_image_copy(),
    //             ImageCopyBuffer {
    //                 buffer: &image_copy_source.buffer,
    //                 layout: ImageDataLayout {
    //                     offset: 0,
    //                     bytes_per_row: Some(
    //                         std::num::NonZeroU32::new(padded_bytes_per_row as u32)
    //                             .unwrap()
    //                             .into(),
    //                     ),
    //                     rows_per_image: None,
    //                 },
    //             },
    //             texture_extent,
    //         );

    //         let render_queue = world.get_resource::<RenderQueue>().unwrap();
    //         render_queue.submit(std::iter::once(encoder.finish()));
    //     }

    //     Ok(())
    // }
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

fn create_image_copier_buffer(render_device: &RenderDevice, (width, height): (u32, u32)) -> Buffer {
    let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row((width) as usize) * 4;

    let buffer = render_device.create_buffer(&BufferDescriptor {
        label: None,
        size: padded_bytes_per_row as u64 * height as u64,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    buffer
}
