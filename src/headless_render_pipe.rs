use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        extract_component::ExtractComponent,
        render_asset::RenderAssetUsages,
        render_resource::{
            Buffer, BufferDescriptor, BufferUsages, Extent3d, TextureDimension, TextureFormat,
            TextureUsages,
        },
        renderer::RenderDevice,
    },
};
use crossbeam_channel::{Receiver, Sender};

use crate::RatatuiRenderStrategy;

pub struct HeadlessRenderPipe {
    pub receiver: Receiver<Vec<u8>>,
    pub receiver_sobel: Option<Receiver<Vec<u8>>>,
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
        let (sender, receiver, buffer, render_handle, cpu_texture) =
            create_image_copy_objects(render_device, images, dimensions);

        let image_copier = ImageCopier::new(render_handle.clone(), buffer, sender);

        let mut image_copier_sobel = None;
        let mut cpu_texture_sobel = None;
        let mut receiver_sobel = None;
        if let RatatuiRenderStrategy::Luminance(ref luminance_config) = strategy {
            if luminance_config.edge_detection {
                let (sender, receiver, buffer, render_handle, cpu_texture) =
                    create_image_copy_objects(render_device, images, dimensions);

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

#[derive(Component, ExtractComponent, Clone, Debug)]
pub struct ImageCopier {
    pub image: Handle<Image>,
    pub buffer: Buffer,
    pub sender: Sender<Vec<u8>>,
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

#[derive(Component, ExtractComponent, Clone, Debug)]
pub struct ImageCopierSobel {
    pub image: Handle<Image>,
    pub buffer: Buffer,
    pub sender: Sender<Vec<u8>>,
}

impl ImageCopierSobel {
    pub fn new(image: Handle<Image>, buffer: Buffer, sender: Sender<Vec<u8>>) -> Self {
        Self {
            image,
            buffer,
            sender,
        }
    }
}

fn create_image_copy_objects(
    render_device: &RenderDevice,
    images: &mut Assets<Image>,
    dimensions: (u32, u32),
) -> (
    Sender<Vec<u8>>,
    Receiver<Vec<u8>>,
    Buffer,
    Handle<Image>,
    Image,
) {
    let (sender, receiver) = crossbeam_channel::unbounded();
    let (render_texture, cpu_texture) = create_render_textures(dimensions);
    let buffer = create_image_copier_buffer(render_device, dimensions);
    let render_handle = images.add(render_texture);

    (sender, receiver, buffer, render_handle, cpu_texture)
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
    let buffer_descriptor = BufferDescriptor {
        label: None,
        size: padded_bytes_per_row as u64 * height as u64,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    };

    render_device.create_buffer(&buffer_descriptor)
}
