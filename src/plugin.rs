use std::io;

use bevy::{
    prelude::*,
    render::{camera::RenderTarget, render_graph::RenderGraph, Render, RenderApp, RenderSet},
    utils::error,
};

use crate::{
    headless::{
        image_copy_source_extract_system, initialize_ratatui_render_context_system_generator,
        receive_rendered_images_system, send_rendered_image_system, ImageCopy, ImageCopyNode,
        RatatuiRenderPipe,
    },
    RatatuiContext, RatatuiRenderWidget,
};

/// Sets up headless rendering and makes the `RatRenderContext` resource available
/// to use in your camera and ratatui draw loop.
///
/// Use `add_render((width, height))` for each render you would like to set up, and a render target
/// and destination image will be created each time, associated with an index (starting at zero).
///
/// Use the renders' indices in the `target(index)` function for a `RenderTarget` that can be
/// placed in a bevy camera.
///
/// Use the index in the `widget(index)` function for a Ratatui widget that will display the output
/// of the render in the terminal.
///
/// Use `print_full_terminal(index)` to add a minimal ratatui draw loop that just draws the render
/// at the given index to the full terminal window.
///
/// example:
/// ```rust
/// app.add_plugins((
///     RatatuiPlugin,
///     RatatuiRenderPlugin::new().add_render((256, 256)).print_full_terminal(0),
/// ))
/// .add_systems(Startup, setup_scene)
///
/// ...
///
/// fn setup_scene(mut commands: Commands, ratatui_render: Res<RatatuiRenderContext>) {
///     commands.spawn(Camera3dBundle {
///         camera: Camera {
///             target: ratatui_render.target(0),
///             ..default()
///         },
///         ..default()
///     });
/// }
/// ```
#[derive(Default)]
pub struct RatatuiRenderPlugin {
    render_configs: Vec<(u32, u32)>,
    print_full_terminal: Option<usize>,
}

impl RatatuiRenderPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_render(mut self, dimensions: (u32, u32)) -> Self {
        self.render_configs.push(dimensions);
        self
    }

    pub fn print_full_terminal(mut self, index: usize) -> Self {
        self.print_full_terminal = Some(index);
        self
    }
}

impl Plugin for RatatuiRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreStartup,
            initialize_ratatui_render_context_system_generator(self.render_configs.clone()),
        )
        .add_systems(First, receive_rendered_images_system);

        let render_app = app.sub_app_mut(RenderApp);

        let mut graph = render_app.world.resource_mut::<RenderGraph>();
        graph.add_node(ImageCopy, ImageCopyNode);
        graph.add_node_edge(bevy::render::graph::CameraDriverLabel, ImageCopy);

        render_app
            .add_systems(ExtractSchedule, image_copy_source_extract_system)
            .add_systems(Render, send_rendered_image_system.after(RenderSet::Render));

        if let Some(index) = self.print_full_terminal {
            app.add_systems(Update, print_full_terminal_system(index).map(error));
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
    pub render_pipes: Vec<RatatuiRenderPipe>,
}

impl RatatuiRenderContext {
    pub fn target(&self, index: usize) -> RenderTarget {
        self.render_pipes[index].target.clone()
    }

    pub fn widget(&self, index: usize) -> RatatuiRenderWidget {
        RatatuiRenderWidget::new(&self.render_pipes[index].image)
    }
}

fn print_full_terminal_system(
    index: usize,
) -> impl FnMut(ResMut<RatatuiContext>, Res<RatatuiRenderContext>) -> io::Result<()> {
    move |mut ratatui, ratatui_render| {
        ratatui.draw(|frame| {
            frame.render_widget(ratatui_render.widget(index), frame.size());
        })?;

        Ok(())
    }
}
