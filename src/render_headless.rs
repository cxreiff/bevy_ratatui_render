// FROM @bugsweeper's BEVY HEADLESS RENDERING EXAMPLE
// (https://github.com/bevyengine/bevy/blob/main/examples/app/headless_renderer.rs)

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
        texture::{BevyDefault, TextureFormatPixelInfo},
        Extract,
    },
};
use crossbeam_channel::{Receiver, Sender};

use crate::render_plugin::RatatuiRenderContext;

/// `ImageCopySource` aggregator in `RenderWorld`
#[derive(Clone, Default, Resource, Deref, DerefMut)]
pub struct ImageCopySources(pub Vec<ImageCopySource>);

/// Used by `ImageCopy` for copying from render target to buffer
#[derive(Clone, Component)]
pub struct ImageCopySource {
    buffer: Buffer,
    enabled: Arc<AtomicBool>,
    src_image: Handle<Image>,
    sender: Sender<Vec<u8>>,
}

impl ImageCopySource {
    pub fn new(
        src_image: Handle<Image>,
        size: Extent3d,
        render_device: &RenderDevice,
        sender: Sender<Vec<u8>>,
    ) -> ImageCopySource {
        let padded_bytes_per_row =
            RenderDevice::align_copy_bytes_per_row((size.width) as usize) * 4;

        let cpu_buffer = render_device.create_buffer(&BufferDescriptor {
            label: None,
            size: padded_bytes_per_row as u64 * size.height as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        ImageCopySource {
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

pub struct RatatuiRenderPipe {
    receiver: Receiver<Vec<u8>>,
    pub target: RenderTarget,
    pub image: Image,
}

impl RatatuiRenderPipe {
    fn new(
        commands: &mut Commands,
        images: &mut ResMut<Assets<Image>>,
        render_device: &Res<RenderDevice>,
        (width, height): (u32, u32),
    ) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();

        let size = Extent3d {
            width,
            height,
            ..Default::default()
        };

        // This is the texture that will be rendered to.
        let mut render_texture = Image::new_fill(
            size,
            TextureDimension::D2,
            &[0; 4],
            TextureFormat::bevy_default(),
            RenderAssetUsages::default(),
        );
        render_texture.texture_descriptor.usage |= TextureUsages::COPY_SRC
            | TextureUsages::RENDER_ATTACHMENT
            | TextureUsages::TEXTURE_BINDING;

        let render_handle = images.add(render_texture);

        // This is the texture that will be copied to.
        let cpu_texture = Image::new_fill(
            size,
            TextureDimension::D2,
            &[0; 4],
            TextureFormat::bevy_default(),
            RenderAssetUsages::default(),
        );

        commands.spawn(ImageCopySource::new(
            render_handle.clone(),
            size,
            render_device,
            sender,
        ));

        Self {
            target: RenderTarget::Image(render_handle),
            image: cpu_texture,
            receiver,
        }
    }
}

/// Creates textures and initializes RatRenderContext
pub fn initialize_ratatui_render_context_system_generator(
    configs: Vec<(u32, u32)>,
) -> impl FnMut(Commands, ResMut<Assets<Image>>, Res<RenderDevice>) {
    move |mut commands, mut images, render_device| {
        let render_pipes = configs
            .iter()
            .map(|dimensions| {
                RatatuiRenderPipe::new(&mut commands, &mut images, &render_device, *dimensions)
            })
            .collect();

        commands.insert_resource(RatatuiRenderContext { render_pipes });
    }
}

/// `RenderGraph` label for `ImageCopy`
#[derive(Debug, PartialEq, Eq, Clone, Hash, RenderLabel)]
pub struct ImageCopy;

/// `RenderGraph` node
#[derive(Default)]
pub struct ImageCopyNode;

impl Node for ImageCopyNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let image_copy_sources = world.get_resource::<ImageCopySources>().unwrap();
        let gpu_images = world.get_resource::<RenderAssets<Image>>().unwrap();

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

            // Calculating correct size of image row because
            // copy_texture_to_buffer can copy image only by rows aligned wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
            // That's why image in buffer can be little bit wider
            // This should be taken into account at copy from buffer stage
            let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(
                (src_image.size.x as usize / block_dimensions.0 as usize) * block_size as usize,
            );

            let texture_extent = Extent3d {
                width: src_image.size.x as u32,
                height: src_image.size.y as u32,
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

/// Extracts `ImageCopySource`s into render world, because `ImageCopy` accesses them
pub fn image_copy_source_extract_system(
    mut commands: Commands,
    image_copy_sources: Extract<Query<&ImageCopySource>>,
) {
    commands.insert_resource(ImageCopySources(
        image_copy_sources
            .iter()
            .cloned()
            .collect::<Vec<ImageCopySource>>(),
    ));
}

/// Sends image from buffer in render world to main world via channel
pub fn send_rendered_image_system(
    image_copy_sources: Res<ImageCopySources>,
    render_device: Res<RenderDevice>,
) {
    for image_copy_source in image_copy_sources.iter() {
        if !image_copy_source.enabled() {
            continue;
        }

        // Finally time to get our data back from the gpu.
        // First we get a buffer slice which represents a chunk of the buffer (which we
        // can't access yet).
        // We want the whole thing so use unbounded range.
        let buffer_slice = image_copy_source.buffer.slice(..);

        // Now things get complicated. WebGPU, for safety reasons, only allows either the GPU
        // or CPU to access a buffer's contents at a time. We need to "map" the buffer which means
        // flipping ownership of the buffer over to the CPU and making access legal. We do this
        // with `BufferSlice::map_async`.
        //
        // The problem is that map_async is not an async function so we can't await it. What
        // we need to do instead is pass in a closure that will be executed when the slice is
        // either mapped or the mapping has failed.
        //
        // The problem with this is that we don't have a reliable way to wait in the main
        // code for the buffer to be mapped and even worse, calling get_mapped_range or
        // get_mapped_range_mut prematurely will cause a panic, not return an error.
        //
        // Using channels solves this as awaiting the receiving of a message from
        // the passed closure will force the outside code to wait. It also doesn't hurt
        // if the closure finishes before the outside code catches up as the message is
        // buffered and receiving will just pick that up.
        //
        // It may also be worth noting that although on native, the usage of asynchronous
        // channels is wholly unnecessary, for the sake of portability to WASM
        // we'll use async channels that work on both native and WASM.

        let (s, r) = crossbeam_channel::bounded(1);

        // Maps the buffer so it can be read on the cpu
        buffer_slice.map_async(MapMode::Read, move |r| match r {
            // This will execute once the gpu is ready, so after the call to poll()
            Ok(r) => s.send(r).expect("Failed to send map update"),
            Err(err) => panic!("Failed to map buffer {err}"),
        });

        // In order for the mapping to be completed, one of three things must happen.
        // One of those can be calling `Device::poll`. This isn't necessary on the web as devices
        // are polled automatically but natively, we need to make sure this happens manually.
        // `Maintain::Wait` will cause the thread to wait on native but not on WebGpu.

        // This blocks until the gpu is done executing everything
        render_device.poll(Maintain::wait()).panic_on_timeout();

        // This blocks until the buffer is mapped
        r.recv().expect("Failed to receive the map_async message");

        // This could fail on app exit, if Main world clears resources (including receiver) while Render world still renders
        let _ = image_copy_source
            .sender
            .send(buffer_slice.get_mapped_range().to_vec());

        // We need to make sure all `BufferView`'s are dropped before we do what we're about
        // to do.
        // Unmap so that we can copy to the staging buffer in the next iteration.
        image_copy_source.buffer.unmap();
    }
}

// Receives image in main world from render world and updates RatRenderContext
pub fn receive_rendered_images_system(mut ratatui_render: ResMut<RatatuiRenderContext>) {
    for render_pipe in &mut ratatui_render.render_pipes {
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
                // shrink data to original image size
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
