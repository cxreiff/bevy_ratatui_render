use bevy::{
    prelude::*,
    render::{camera::RenderTarget, extract_component::ExtractComponent, renderer::RenderDevice},
};
use bevy_ratatui::{event::ResizeEvent, terminal::RatatuiContext};

use crate::image_pipe::{create_image_pipe, ImageReceiver, ImageSender};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(PostStartup, initial_autoresize_system)
        .add_systems(PreUpdate, spawn_ratatui_camera_machinery_system)
        .add_systems(Update, autoresize_ratatui_camera_system);
}

#[derive(Component)]
#[require(RatatuiCameraStrategy)]
pub struct RatatuiCamera {
    dimensions: (u32, u32),
    autoresize: bool,
    autoresize_function: fn((u32, u32)) -> (u32, u32),
}

impl Default for RatatuiCamera {
    fn default() -> Self {
        Self {
            dimensions: (256, 256),
            autoresize: false,
            autoresize_function: |(width, height)| (width * 2, height * 2),
        }
    }
}

#[derive(Component, Default)]
pub enum RatatuiCameraStrategy {
    #[default]
    HalfBlock,
    Luminance,
}

#[derive(Component, ExtractComponent, Clone, Deref, DerefMut)]
pub struct RatatuiCameraSender(ImageSender);

#[derive(Component, Deref, DerefMut)]
pub struct RatatuiCameraReceiver(ImageReceiver);

#[derive(Component, ExtractComponent, Clone, Deref, DerefMut)]
pub struct RatatuiSobelSender(ImageSender);

#[derive(Component, Deref, DerefMut)]
pub struct RatatuiSobelReceiver(ImageReceiver);

fn spawn_ratatui_camera_machinery_system(
    mut commands: Commands,
    mut ratatui_cameras: Query<
        (Entity, &mut Camera, &RatatuiCamera, &RatatuiCameraStrategy),
        Added<RatatuiCamera>,
    >,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    for (entity, mut camera, ratatui_camera, strategy) in &mut ratatui_cameras {
        let mut entity = commands.entity(entity);

        let (sender, receiver) =
            create_image_pipe(&mut images, &render_device, ratatui_camera.dimensions);

        // TODO: Can we skip this line and just modify the render graph?
        camera.target = RenderTarget::from(sender.sender_image.clone());

        entity.insert((RatatuiCameraSender(sender), RatatuiCameraReceiver(receiver)));

        match strategy {
            RatatuiCameraStrategy::HalfBlock => {}
            RatatuiCameraStrategy::Luminance => {
                let (sender, receiver) =
                    create_image_pipe(&mut images, &render_device, ratatui_camera.dimensions);
                entity.insert((RatatuiSobelSender(sender), RatatuiSobelReceiver(receiver)));
            }
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
        &mut RatatuiCameraReceiver,
        Option<&mut RatatuiSobelSender>,
        Option<&mut RatatuiSobelReceiver>,
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
            mut camera_receiver,
            sobel_sender,
            sobel_receiver,
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
                camera_receiver.0 = receiver;

                if let (Some(mut sobel_sender), Some(mut sobel_receiver)) =
                    (sobel_sender, sobel_receiver)
                {
                    let (sender, receiver) =
                        create_image_pipe(&mut images, &render_device, new_dimensions);

                    sobel_sender.0 = sender;
                    sobel_receiver.0 = receiver;
                }
            }
        }
    }
}
