use std::io;

use bevy::{
    prelude::*,
    render::{camera::RenderTarget, render_graph::RenderGraph, Render, RenderApp, RenderSet},
    utils::error,
};

use crate::{
    render_headless::{
        image_copy_source_extract_system, initialize_ratatui_render_context_system,
        receive_rendered_image_system, send_rendered_image_system, ImageCopy, ImageCopyNode,
        MainWorldReceiver, RenderWorldSender,
    },
    RatContext, RatRenderWidget,
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
            .add_systems(PreStartup, initialize_ratatui_render_context_system)
            .add_systems(First, receive_rendered_image_system);

        let render_app = app.sub_app_mut(RenderApp);

        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(ImageCopy, ImageCopyNode);
        graph.add_node_edge(bevy::render::graph::CameraDriverLabel, ImageCopy);

        render_app
            .insert_resource(RenderWorldSender(s))
            .add_systems(ExtractSchedule, image_copy_source_extract_system)
            .add_systems(Render, send_rendered_image_system.after(RenderSet::Render));

        if self.print_full_terminal {
            app.add_systems(Update, print_full_terminal_system.map(error));
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
    pub rendered_image: Image,
}

impl RatRenderContext {
    pub fn target(&self) -> RenderTarget {
        self.camera_target.clone()
    }

    pub fn widget(&self) -> RatRenderWidget {
        RatRenderWidget::new(&self.rendered_image)
    }
}

pub fn print_full_terminal_system(
    mut rat: ResMut<RatContext>,
    rat_render_context: Res<RatRenderContext>,
) -> io::Result<()> {
    rat.draw(|frame| {
        frame.render_widget(rat_render_context.widget(), frame.size());
    })?;

    Ok(())
}
