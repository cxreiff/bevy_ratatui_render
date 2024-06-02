use std::io;

use bevy::{
    prelude::*,
    render::{camera::RenderTarget, render_graph::RenderGraph, Render, RenderApp, RenderSet},
    utils::error,
};

use crate::{
    rat_plugin::RatatuiContext,
    render_headless::{
        image_copy_source_extract_system, initialize_ratatui_render_context_system_generator,
        receive_rendered_image_system, send_rendered_image_system, ImageCopy, ImageCopyNode,
        MainWorldReceiver, RenderWorldSender,
    },
    render_widget::RatatuiRenderWidget,
};

/// Sets up headless rendering and makes the `RatRenderContext` resource available
/// to use in your camera and ratatui draw loop.
///
/// Use `print_full_terminal()` to add a minimal ratatui draw loop that just draws
/// your bevy scene to the full terminal window.
///
/// basic setup:
///
/// ```
/// app.add_plugins((
///     RatatuiPlugin,
///     RatatuiRenderPlugin::new(512, 512).print_full_terminal(),
/// ))
/// .add_systems(Startup, setup_camera)
///
/// ...
///
/// fn setup_camera(mut commands: Commands, rat_render: Res<RatatuiRenderContext>) {
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
pub struct RatatuiRenderPlugin {
    width: u32,
    height: u32,
    print_full_terminal: bool,
}

impl RatatuiRenderPlugin {
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

impl Plugin for RatatuiRenderPlugin {
    fn build(&self, app: &mut App) {
        let (s, r) = crossbeam_channel::unbounded();

        app.insert_resource(MainWorldReceiver(r))
            .add_systems(
                PreStartup,
                initialize_ratatui_render_context_system_generator(self.width, self.height),
            )
            .add_systems(First, receive_rendered_image_system);

        let render_app = app.sub_app_mut(RenderApp);

        let mut graph = render_app.world.resource_mut::<RenderGraph>();
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

/// Resource containing a bevy camera render target and an image that will be updated each frame
/// with the results of whatever is rendered to that target.
///
/// `target()` to clone the render target.
///
/// `widget()` to generate a ratatui widget that will draw whatever was rendered to the render
/// target in the ratatui frame.
#[derive(Resource, Default)]
pub struct RatatuiRenderContext {
    pub camera_target: RenderTarget,
    pub rendered_image: Image,
}

impl RatatuiRenderContext {
    pub fn target(&self) -> RenderTarget {
        self.camera_target.clone()
    }

    pub fn widget(&self) -> RatatuiRenderWidget {
        RatatuiRenderWidget::new(&self.rendered_image)
    }
}

fn print_full_terminal_system(
    mut rat: ResMut<RatatuiContext>,
    rat_render_context: Res<RatatuiRenderContext>,
) -> io::Result<()> {
    rat.draw(|frame| {
        frame.render_widget(rat_render_context.widget(), frame.size());
    })?;

    Ok(())
}
