use bevy::{
    core_pipeline::prepass::{DepthPrepass, NormalPrepass},
    image::TextureFormatPixelInfo,
    prelude::*,
    render::{
        camera::RenderTarget,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::{Buffer, Maintain, MapMode},
        renderer::RenderDevice,
        Render, RenderApp, RenderSet,
    },
};
use bevy_ratatui::{event::ResizeEvent, terminal::RatatuiContext};
use crossbeam_channel::Sender;

use crate::{
    camera_image_pipe::{create_image_pipe, ImageReceiver, ImageSender},
    RatatuiCamera, RatatuiCameraStrategy, RatatuiCameraWidget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        ExtractComponentPlugin::<RatatuiCameraSender>::default(),
        ExtractComponentPlugin::<RatatuiSobelSender>::default(),
    ))
    .add_systems(PostStartup, initial_autoresize_system)
    .add_systems(
        First,
        (
            spawn_ratatui_camera_machinery_system,
            receive_camera_images_system,
        ),
    )
    .add_systems(Update, autoresize_ratatui_camera_system);

    let render_app = app.sub_app_mut(RenderApp);
    render_app.add_systems(
        Render,
        (send_camera_images_system, send_sobel_images_system).after(RenderSet::Render),
    );
}

#[derive(Component, ExtractComponent, Clone, Deref, DerefMut)]
pub struct RatatuiCameraSender(ImageSender);

#[derive(Component, ExtractComponent, Clone, Deref, DerefMut)]
pub struct RatatuiSobelSender(ImageSender);

fn spawn_ratatui_camera_machinery_system(
    mut commands: Commands,
    mut ratatui_cameras: Query<(Entity, &mut Camera, &RatatuiCamera), Added<RatatuiCamera>>,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    for (entity_id, mut camera, ratatui_camera) in &mut ratatui_cameras {
        let mut entity = commands.entity(entity_id);

        let (sender, receiver) =
            create_image_pipe(&mut images, &render_device, ratatui_camera.dimensions);

        // TODO: Can we skip this line and just modify the render graph?
        camera.target = RenderTarget::from(sender.sender_image.clone());

        entity.insert(RatatuiCameraSender(sender));

        let mut widget = RatatuiCameraWidget {
            camera_receiver: receiver,
            sobel_receiver: None,
            strategy: ratatui_camera.strategy.clone(),
        };

        match ratatui_camera.strategy {
            RatatuiCameraStrategy::HalfBlocks => {}
            RatatuiCameraStrategy::Luminance(_) => {
                let (sender, receiver) =
                    create_image_pipe(&mut images, &render_device, ratatui_camera.dimensions);
                entity.insert((RatatuiSobelSender(sender), DepthPrepass, NormalPrepass));
                widget.sobel_receiver = Some(receiver);
            }
        }

        entity.insert(widget);
    }
}

fn send_camera_images_system(
    ratatui_camera_senders: Query<&RatatuiCameraSender>,
    render_device: Res<RenderDevice>,
) {
    for camera_sender in &ratatui_camera_senders {
        send_image_buffer(&render_device, &camera_sender.buffer, &camera_sender.sender);
    }
}

fn send_sobel_images_system(
    ratatui_sobel_senders: Query<&RatatuiSobelSender>,
    render_device: Res<RenderDevice>,
) {
    for sobel_sender in &ratatui_sobel_senders {
        send_image_buffer(&render_device, &sobel_sender.buffer, &sobel_sender.sender);
    }
}

fn send_image_buffer(render_device: &RenderDevice, buffer: &Buffer, sender: &Sender<Vec<u8>>) {
    let buffer_slice = buffer.slice(..);

    let (s, r) = crossbeam_channel::bounded(1);

    buffer_slice.map_async(MapMode::Read, move |r| match r {
        Ok(r) => s.send(r).expect("failed to send map update"),
        Err(err) => panic!("failed to map buffer: {err}"),
    });

    render_device.poll(Maintain::wait()).panic_on_timeout();

    r.recv().expect("failed to receive the map_async message");

    let _ = sender.send(buffer_slice.get_mapped_range().to_vec());

    buffer.unmap();
}

fn receive_camera_images_system(mut ratatui_camera_widgets: Query<&mut RatatuiCameraWidget>) {
    for mut ratatui_camera_widget in &mut ratatui_camera_widgets {
        receive_image(&mut ratatui_camera_widget.camera_receiver);

        if let Some(ref mut sobel_receiver) = ratatui_camera_widget.sobel_receiver {
            receive_image(sobel_receiver);
        }
    }
}

fn receive_image(image_receiver: &mut ImageReceiver) {
    let mut image_data = Vec::new();
    while let Ok(data) = image_receiver.receiver.try_recv() {
        image_data = data;
    }

    if !image_data.is_empty() {
        let row_bytes = image_receiver.receiver_image.width() as usize
            * image_receiver
                .receiver_image
                .texture_descriptor
                .format
                .pixel_size();

        let aligned_row_bytes = RenderDevice::align_copy_bytes_per_row(row_bytes);

        if row_bytes == aligned_row_bytes {
            image_receiver.receiver_image.data.clone_from(&image_data);
        } else {
            image_receiver.receiver_image.data = image_data
                .chunks(aligned_row_bytes)
                .take(image_receiver.receiver_image.height() as usize)
                .flat_map(|row| &row[..row_bytes.min(row.len())])
                .cloned()
                .collect();
        }
    }
}

/// Sends a single resize event during startup.
fn initial_autoresize_system(
    ratatui: Res<RatatuiContext>,
    mut resize_events: EventWriter<ResizeEvent>,
) {
    if let Ok(size) = ratatui.size() {
        resize_events.send(ResizeEvent(size));
    }
}

/// Autoresizes the send/receive textures to fit the terminal dimensions.
fn autoresize_ratatui_camera_system(
    mut ratatui_cameras: Query<(
        &mut Camera,
        &mut RatatuiCamera,
        &mut RatatuiCameraSender,
        Option<&mut RatatuiSobelSender>,
        &mut RatatuiCameraWidget,
    )>,
    mut resize_events: EventReader<ResizeEvent>,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    if let Some(ResizeEvent(dimensions)) = resize_events.read().last() {
        for (
            mut camera,
            mut ratatui_camera,
            mut camera_sender,
            sobel_sender,
            mut ratatui_camera_widget,
        ) in &mut ratatui_cameras
        {
            if ratatui_camera.autoresize {
                let terminal_dimensions = (dimensions.width as u32, dimensions.height as u32 * 2);
                let new_dimensions = (ratatui_camera.autoresize_function)(terminal_dimensions);
                ratatui_camera.dimensions = new_dimensions;

                let (sender, receiver) =
                    create_image_pipe(&mut images, &render_device, new_dimensions);

                // TODO: Can we skip this line and just modify the render graph?
                camera.target = RenderTarget::from(sender.sender_image.clone());

                camera_sender.0 = sender;
                ratatui_camera_widget.camera_receiver = receiver;

                if let Some(mut sobel_sender) = sobel_sender {
                    let (sender, receiver) =
                        create_image_pipe(&mut images, &render_device, new_dimensions);

                    sobel_sender.0 = sender;
                    ratatui_camera_widget.sobel_receiver = Some(receiver);
                }
            }
        }
    }
}
