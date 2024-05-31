use bevy::{
    prelude::*,
    render::{
        render_graph::RenderGraph, render_resource::Extent3d, renderer::RenderDevice, Render,
        RenderApp, RenderSet,
    },
    utils::error,
};
use image::DynamicImage;

use crate::{
    rat_print, rat_receive,
    render_headless::{
        create_render_textures, image_copy_extract, receive_image_from_buffer, ImageCopier,
        ImageCopy, ImageCopyNode, ImageToSave, MainWorldReceiver, RenderWorldSender,
    },
};

/// basic setup:
///
/// ```
/// app.add_plugins((
///     RatPlugin,
///     RatRenderPlugin::new(1024, 1024).print_full_terminal(),
/// ))
/// .add_systems(Startup, rat_create.pipe(setup_camera))
///
/// ...
///
/// fn setup_camera(In(target): In<RatCreateOutput>, mut commands: Commands) {
///     commands.spawn(Camera3dBundle {
///         camera: Camera {
///             target,
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
            .add_systems(PreStartup, setup_rendering)
            .add_systems(Update, rat_receive);

        let render_app = app.sub_app_mut(RenderApp);

        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(ImageCopy, ImageCopyNode);
        graph.add_node_edge(bevy::render::graph::CameraDriverLabel, ImageCopy);

        render_app
            .insert_resource(RenderWorldSender(s))
            .add_systems(ExtractSchedule, image_copy_extract)
            .add_systems(Render, receive_image_from_buffer.after(RenderSet::Render));

        if self.print_full_terminal {
            app.add_systems(Update, rat_receive.pipe(rat_print).map(error));
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
    width: u32,
    height: u32,
}

impl RatRenderConfig {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Resource, Default)]
pub struct RatRenderContext {
    pub camera_target: Handle<Image>,
    pub rendered_image: Option<DynamicImage>,
}

fn setup_rendering(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    rat_render_config: ResMut<RatRenderConfig>,
    render_device: Res<RenderDevice>,
) {
    let size = Extent3d {
        width: rat_render_config.width,
        height: rat_render_config.height,
        ..Default::default()
    };

    let (render_texture, cpu_texture) = create_render_textures(size);
    let render_handle = images.add(render_texture);
    let cpu_handle = images.add(cpu_texture);

    commands.spawn(ImageCopier::new(
        render_handle.clone(),
        size,
        &render_device,
    ));
    commands.spawn(ImageToSave(cpu_handle));

    commands.insert_resource(RatRenderContext {
        camera_target: render_handle,
        rendered_image: None,
    });
}
