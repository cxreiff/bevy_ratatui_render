use bevy::{
    core_pipeline::prepass::{DepthPrepass, NormalPrepass},
    prelude::*,
    render::{
        camera::RenderTarget,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        renderer::RenderDevice,
        Render, RenderApp, RenderSet,
    },
};
use bevy_ratatui::{event::ResizeEvent, terminal::RatatuiContext};

use crate::{
    camera_image_pipe::{
        create_image_pipe, receive_image, send_image_buffer, ImageReceiver, ImageSender,
    },
    RatatuiCamera, RatatuiCameraEdgeDetection, RatatuiCameraWidget,
};

// TODO: Handle readback cleanup when user-facing components are removed.

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        ExtractComponentPlugin::<RatatuiCameraSender>::default(),
        ExtractComponentPlugin::<RatatuiSobelSender>::default(),
    ))
    .add_systems(PostStartup, initial_autoresize_system)
    .add_systems(
        PreUpdate,
        (
            autoresize_ratatui_camera_system,
            (
                spawn_ratatui_camera_readback_system,
                spawn_ratatui_sobel_readback_system,
                receive_camera_images_system,
                receive_sobel_images_system,
            ),
            create_ratatui_camera_widgets_system,
        )
            .chain(),
    );

    let render_app = app.sub_app_mut(RenderApp);
    render_app.add_systems(
        Render,
        (send_camera_images_system, send_sobel_images_system).after(RenderSet::Render),
    );
}

#[derive(Component, ExtractComponent, Clone, Deref, DerefMut)]
pub struct RatatuiCameraSender(ImageSender);

#[derive(Component, Deref, DerefMut)]
pub struct RatatuiCameraReceiver(ImageReceiver);

#[derive(Component, ExtractComponent, Clone, Deref, DerefMut)]
pub struct RatatuiSobelSender(ImageSender);

#[derive(Component, Deref, DerefMut)]
pub struct RatatuiSobelReceiver(ImageReceiver);

fn spawn_ratatui_camera_readback_system(
    mut commands: Commands,
    mut ratatui_cameras: Query<
        (Entity, &mut Camera, &RatatuiCamera),
        Or<(Added<RatatuiCamera>, Changed<RatatuiCamera>)>,
    >,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    for (entity_id, mut camera, ratatui_camera) in &mut ratatui_cameras {
        let mut entity = commands.entity(entity_id);

        let (sender, receiver) =
            create_image_pipe(&mut images, &render_device, ratatui_camera.dimensions);

        camera.target = RenderTarget::from(sender.sender_image.clone());

        entity.insert((RatatuiCameraSender(sender), RatatuiCameraReceiver(receiver)));
    }
}

fn spawn_ratatui_sobel_readback_system(
    mut commands: Commands,
    mut ratatui_cameras: Query<
        (Entity, &RatatuiCamera),
        Or<(
            Added<RatatuiCameraEdgeDetection>,
            (With<RatatuiCameraEdgeDetection>, Changed<RatatuiCamera>),
        )>,
    >,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    for (entity_id, ratatui_camera) in &mut ratatui_cameras {
        let mut entity = commands.entity(entity_id);

        let (sender, receiver) =
            create_image_pipe(&mut images, &render_device, ratatui_camera.dimensions);

        entity.insert((
            RatatuiSobelSender(sender),
            RatatuiSobelReceiver(receiver),
            DepthPrepass,
            NormalPrepass,
            Msaa::Off,
        ));
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

fn receive_camera_images_system(mut camera_receivers: Query<&mut RatatuiCameraReceiver>) {
    for mut camera_receiver in &mut camera_receivers {
        receive_image(&mut camera_receiver);
    }
}

fn receive_sobel_images_system(mut sobel_receivers: Query<&mut RatatuiSobelReceiver>) {
    for mut sobel_receiver in &mut sobel_receivers {
        receive_image(&mut sobel_receiver);
    }
}

fn create_ratatui_camera_widgets_system(
    mut commands: Commands,
    ratatui_cameras: Query<(
        Entity,
        &RatatuiCamera,
        Option<&RatatuiCameraEdgeDetection>,
        &RatatuiCameraReceiver,
        Option<&RatatuiSobelReceiver>,
    )>,
) {
    for (entity_id, ratatui_camera, edge_detection, camera_receiver, sobel_receiver) in
        &ratatui_cameras
    {
        let mut entity = commands.entity(entity_id);

        let camera_image = match camera_receiver.receiver_image.clone().try_into_dynamic() {
            Ok(image) => image,
            Err(e) => panic!("failed to create camera image buffer {e:?}"),
        };

        let sobel_image = sobel_receiver.as_ref().map(|image_sobel| {
            match image_sobel.receiver_image.clone().try_into_dynamic() {
                Ok(image) => image,
                Err(e) => panic!("failed to create sobel image buffer {e:?}"),
            }
        });

        let widget = RatatuiCameraWidget {
            camera_image,
            sobel_image,
            strategy: ratatui_camera.strategy.clone(),
            edge_detection: edge_detection.cloned(),
        };

        entity.insert(widget);
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
    mut ratatui_cameras: Query<&mut RatatuiCamera>,
    mut resize_events: EventReader<ResizeEvent>,
) {
    if let Some(ResizeEvent(dimensions)) = resize_events.read().last() {
        for mut ratatui_camera in &mut ratatui_cameras {
            if ratatui_camera.autoresize {
                let terminal_dimensions = (dimensions.width as u32, dimensions.height as u32 * 2);
                let new_dimensions = (ratatui_camera.autoresize_function)(terminal_dimensions);
                ratatui_camera.dimensions = new_dimensions;
            }
        }
    }
}
