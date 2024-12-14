use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        render_resource::{
            Buffer, BufferDescriptor, BufferUsages, Extent3d, TextureDimension, TextureFormat,
            TextureUsages,
        },
        renderer::RenderDevice,
    },
};
use crossbeam_channel::{Receiver, Sender};

#[derive(Clone)]
pub struct ImageSender {
    pub sender: Sender<Vec<u8>>,
    pub sender_image: Handle<Image>,
    pub buffer: Buffer,
}

pub struct ImageReceiver {
    pub receiver: Receiver<Vec<u8>>,
    pub receiver_image: Image,
}

pub fn create_image_pipe(
    images: &mut Assets<Image>,
    render_device: &RenderDevice,
    dimensions: (u32, u32),
) -> (ImageSender, ImageReceiver) {
    let (sender, receiver, buffer, sender_image, receiver_image) =
        create_image_copy_objects(render_device, images, dimensions);

    let camera_sender = ImageSender {
        sender,
        sender_image,
        buffer,
    };

    let camera_receiver = ImageReceiver {
        receiver,
        receiver_image,
    };

    (camera_sender, camera_receiver)
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
    let (sender_texture, receiver_texture) = create_image_copy_textures(dimensions);
    let buffer = create_image_copy_buffer(render_device, dimensions);
    let sender_handle = images.add(sender_texture);

    (sender, receiver, buffer, sender_handle, receiver_texture)
}

fn create_image_copy_textures(dimensions: (u32, u32)) -> (Image, Image) {
    let (width, height) = dimensions;
    let size = Extent3d {
        width,
        height,
        ..Default::default()
    };

    let mut sender_texture = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0; 4],
        TextureFormat::bevy_default(),
        RenderAssetUsages::default(),
    );

    let receiver_texture = sender_texture.clone();

    sender_texture.texture_descriptor.usage |=
        TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;

    (sender_texture, receiver_texture)
}

fn create_image_copy_buffer(render_device: &RenderDevice, (width, height): (u32, u32)) -> Buffer {
    let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row((width) as usize) * 4;
    let buffer_descriptor = BufferDescriptor {
        label: None,
        size: padded_bytes_per_row as u64 * height as u64,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    };

    render_device.create_buffer(&buffer_descriptor)
}
