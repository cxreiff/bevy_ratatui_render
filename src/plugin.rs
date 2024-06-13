use std::io;

use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget, render_graph::RenderGraph, renderer::RenderDevice, Render, RenderApp,
        RenderSet,
    },
    utils::{error, hashbrown::HashMap},
};
use bevy_ratatui::terminal::RatatuiContext;

use crate::{
    headless::{
        image_copier_extract_system, receive_rendered_images_system, send_rendered_image_system,
        HeadlessRenderPipe, ImageCopy, ImageCopyNode,
    },
    RatatuiRenderWidget,
};

/// Sets up headless rendering and makes the `RatatuiRenderContext` resource available
/// to use in your camera and ratatui draw loop.
///
/// Can be added multiple times to set up multiple render targets. Use
/// `RatatuiRenderPlugin::new("id", (width, height))` for each render you would like to set up,
/// and then pass your string id into the `RatatuiRenderContext` resource's `target(id)` and
/// `widget(id)` methods for the render target and ratatui widget respectively.
///
/// Place the render target in a bevy camera, and use the ratatui widget in a ratatui draw loop in
/// order to display the bevy camera's render in the terminal.
///
/// Use `print_full_terminal()` to add a minimal ratatui draw loop that just draws the render
/// to the full terminal window.
///
/// # example:
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_ratatui::RatatuiPlugins;
/// # use bevy_ratatui_render::{RatatuiRenderContext, RatatuiRenderPlugin};
/// #
/// fn main() {
///     App::new()
///         .add_plugins((
///             DefaultPlugins,
///             RatatuiPlugins::default(),
///             RatatuiRenderPlugin::new("main", (256, 256)).print_full_terminal(),
///         ))
///         .add_systems(Startup, setup_scene);
/// }
///
/// // ...
///
/// fn setup_scene(mut commands: Commands, ratatui_render: Res<RatatuiRenderContext>) {
///     commands.spawn(Camera3dBundle {
///         camera: Camera {
///             target: ratatui_render.target("main").unwrap(),
///             ..default()
///         },
///         ..default()
///     });
/// }
/// ```
pub struct RatatuiRenderPlugin {
    label: String,
    dimensions: (u32, u32),
    print_full_terminal: bool,
    disabled: bool,
}

impl RatatuiRenderPlugin {
    /// Create an instance of RatatuiRenderPlugin.
    ///
    /// * `label` - Unique descriptive identifier. To access the render target and ratatui widget
    /// created by this instance of the plugin, pass the same string into the `target(id)` and
    /// `widget(id)` methods on the `RatatuiRenderContext` resource.
    ///
    /// * `dimensions` - (width, height) - the dimensions of the texture that will be rendered to.
    pub fn new(label: &str, dimensions: (u32, u32)) -> Self {
        Self {
            label: label.into(),
            dimensions,
            print_full_terminal: false,
            disabled: false,
        }
    }

    /// Initializes RatatuiRenderContext resource but skips setting up the headless rendering.
    /// `target(id)` and `widget(id)` on the context resource will each return None.
    ///
    /// Working on a bevy application that renders to the terminal, you may occasionally want to
    /// see your application running in a normal window for debugging or convenience. Calling this
    /// method on the plugin allows you to test your bevy app in a window without being forced to
    /// comment out every bevy system with `Res<RatatuiRenderContext>` as a parameter.
    ///
    /// Refer to the `disable` example for a bevy app that gracefully falls back to a normal window
    /// when `disabled()` is used (for example, passing along normal bevy input events to your
    /// terminal keyboard event handlers).
    pub fn disable(mut self) -> Self {
        self.disabled = true;
        self
    }

    /// Adds a bevy system that draws the ratatui widget containing your bevy application's render
    /// output to the full terminal each frame (preserving aspect ratio). If you don't need to
    /// customize the ratatui draw loop, use this to cut out some boilerplate.
    pub fn print_full_terminal(mut self) -> Self {
        self.print_full_terminal = true;
        self
    }
}

impl Plugin for RatatuiRenderPlugin {
    fn build(&self, app: &mut App) {
        if self.disabled {
            app.init_resource::<RatatuiRenderContext>();
            return;
        }

        if app
            .world
            .get_resource_mut::<RatatuiRenderContext>()
            .is_none()
        {
            app.init_resource::<RatatuiRenderContext>()
                .add_systems(First, receive_rendered_images_system);

            let render_app = app.sub_app_mut(RenderApp);

            let mut graph = render_app.world.resource_mut::<RenderGraph>();
            graph.add_node(ImageCopy, ImageCopyNode);
            graph.add_node_edge(bevy::render::graph::CameraDriverLabel, ImageCopy);

            render_app
                .add_systems(ExtractSchedule, image_copier_extract_system)
                .add_systems(Render, send_rendered_image_system.after(RenderSet::Render));
        }

        app.add_systems(
            PreStartup,
            initialize_context_system_generator(self.label.clone(), self.dimensions),
        );

        if self.print_full_terminal {
            app.add_systems(
                Update,
                print_full_terminal_system(self.label.clone()).map(error),
            );
        }
    }

    fn is_unique(&self) -> bool {
        false
    }
}

/// Resource containing a bevy camera render target and an image that will be updated each frame
/// with the results of whatever is rendered to that target.
///
/// `target(id)` to clone the render target.
///
/// `widget(id)` to generate a ratatui widget that will draw whatever was rendered to the render
/// target in the ratatui frame.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct RatatuiRenderContext(HashMap<String, HeadlessRenderPipe>);

impl RatatuiRenderContext {
    /// Gets a clone of the render target, for placement inside a bevy camera.
    ///
    /// * `id` - Unique descriptive identifier, must match the id provided when the corresponding
    /// `RatatuiRenderPlugin` was instantiated.
    pub fn target(&self, id: &str) -> Option<RenderTarget> {
        let pipe = self.get(id)?;
        Some(pipe.target.clone())
    }

    /// Gets a ratatui widget, that when drawn will print the most recent image rendered to the
    /// render target of the same id.
    ///
    /// * `id` - Unique descriptive identifier, must match the id provided when the corresponding
    /// `RatatuiRenderPlugin` was instantiated.
    pub fn widget(&self, id: &str) -> Option<RatatuiRenderWidget> {
        let pipe = self.get(id)?;
        Some(RatatuiRenderWidget::new(&pipe.image))
    }
}

/// Creates a headless render pipe and adds it to the RatatuiRenderContext resource.
fn initialize_context_system_generator(
    label: String,
    dimensions: (u32, u32),
) -> impl FnMut(Commands, ResMut<Assets<Image>>, Res<RenderDevice>, ResMut<RatatuiRenderContext>) {
    move |mut commands, mut images, render_device, mut context| {
        context.insert(
            label.clone(),
            HeadlessRenderPipe::new(&mut commands, &mut images, &render_device, dimensions),
        );
    }
}

/// Draws the widget for the provided id in the full terminal, each frame.
fn print_full_terminal_system(
    id: String,
) -> impl FnMut(ResMut<RatatuiContext>, Res<RatatuiRenderContext>) -> io::Result<()> {
    move |mut ratatui, ratatui_render| {
        if let Some(render_widget) = ratatui_render.widget(&id) {
            ratatui.draw(|frame| {
                frame.render_widget(render_widget, frame.size());
            })?;
        }

        Ok(())
    }
}
