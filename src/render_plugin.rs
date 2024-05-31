use bevy::{
    prelude::*,
    render::{camera::RenderTarget, render_graph::RenderGraph, Render, RenderApp, RenderSet},
    utils::error,
};
use image::DynamicImage;

use crate::{
    render_headless::{
        image_copy_extract, receive_image_from_buffer, ImageCopy, ImageCopyNode, MainWorldReceiver,
        RenderWorldSender,
    },
    render_systems::{rat_create, rat_print, rat_receive},
    RatRenderWidget,
};

/// basic setup:
///
/// ```
/// app.add_plugins((
///     RatPlugin,
///     RatRenderPlugin::new(512, 512).print_full_terminal(),
/// ))
/// .add_systems(Startup, setup_camera)
///
/// ...
///
/// fn setup_camera(mut commands: Commands, rat_render: Res<RatRenderContext>) {
///     commands.spawn(Camera3dBundle {
///         camera: Camera {
///             target: rat_render.target(),
///             ..default()
///         },
///         ..default()
///     });
/// }
/// ```
#[derive(Default)]
pub struct RatRenderPlugin {
    width: u32,
    height: u32,
    print_full_terminal: bool,
}

impl Plugin for RatRenderPlugin {
    fn build(&self, app: &mut App) {
        let (s, r) = crossbeam_channel::unbounded();

        app.insert_resource(MainWorldReceiver(r))
            .insert_resource(RatRenderConfig::new(self.width, self.height))
            .add_systems(PreStartup, rat_create)
            .add_systems(First, rat_receive);

        let render_app = app.sub_app_mut(RenderApp);

        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(ImageCopy, ImageCopyNode);
        graph.add_node_edge(bevy::render::graph::CameraDriverLabel, ImageCopy);

        render_app
            .insert_resource(RenderWorldSender(s))
            .add_systems(ExtractSchedule, image_copy_extract)
            .add_systems(Render, receive_image_from_buffer.after(RenderSet::Render));

        if self.print_full_terminal {
            app.add_systems(Update, rat_print.map(error));
        }
    }
}

impl RatRenderPlugin {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..default()
        }
    }

    pub fn print_full_terminal(mut self) -> Self {
        self.print_full_terminal = true;
        self
    }
}

#[derive(Resource)]
pub struct RatRenderConfig {
    pub width: u32,
    pub height: u32,
}

impl RatRenderConfig {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Resource, Default)]
pub struct RatRenderContext {
    pub camera_target: RenderTarget,
    pub rendered_image: Option<DynamicImage>,
}

impl RatRenderContext {
    pub fn target(&self) -> RenderTarget {
        self.camera_target.clone()
    }

    pub fn widget(&self) -> RatRenderWidget {
        RatRenderWidget::new(&self.rendered_image)
    }
}
