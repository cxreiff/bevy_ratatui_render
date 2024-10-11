use std::io;

use bevy::{
    ecs::system::{RunSystemOnce, SystemState},
    prelude::*,
    render::{
        camera::RenderTarget, render_graph::RenderGraph, renderer::RenderDevice, Render, RenderApp,
        RenderSet,
    },
    utils::{error, hashbrown::HashMap},
};
use bevy_ratatui::{event::ResizeEvent, terminal::RatatuiContext};

use crate::{
    headless::{
        image_copier_extract_system, receive_rendered_images_system, send_rendered_image_system,
        HeadlessRenderPipe, ImageCopier, ImageCopy, ImageCopyNode,
    },
    RatatuiRenderWidget,
};

/// Function that converts terminal dimensions to render texture dimensions.
pub type AutoresizeConversionFn = fn((u32, u32)) -> (u32, u32);

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
/// Use `autoresize()` to automatically match the render image to the terminal dimensions during
/// startup and when the terminal is resized.
///
/// # example:
/// ```no_run
/// # use std::time::Duration;
/// # use bevy::app::ScheduleRunnerPlugin;
/// # use bevy::winit::WinitPlugin;
/// # use bevy::prelude::*;
/// # use bevy_ratatui::RatatuiPlugins;
/// # use bevy_ratatui_render::{RatatuiRenderContext, RatatuiRenderPlugin};
/// #
/// fn main() {
///     App::new()
///         .add_plugins((
///             // Disable WinitPlugin to avoid a panic in environments without a display server.
///             DefaultPlugins.build().disable::<WinitPlugin>(),
///
///             // Create windowless loop and set its duration per frame (inverse of frame rate).
///             ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 60.)),
///
///             // RatatuiPlugins sets up the Ratatui context and forwards input events.
///             RatatuiPlugins::default(),
///
///             // RatatuiRenderPlugin connects a bevy camera target to a ratatui widget.
///             RatatuiRenderPlugin::new("main", (256, 256)).print_full_terminal().autoresize(),
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
    id: String,
    dimensions: (u32, u32),
    print_full_terminal: bool,
    autoresize: bool,
    autoresize_conversion_fn: Option<AutoresizeConversionFn>,
    disabled: bool,
}

impl RatatuiRenderPlugin {
    /// Create an instance of RatatuiRenderPlugin.
    ///
    /// * `id` - Unique descriptive identifier. To access the render target and ratatui widget
    ///   created by this instance of the plugin, pass the same string into the `target(id)` and
    ///   `widget(id)` methods on the `RatatuiRenderContext` resource.
    ///
    /// * `dimensions` - (width, height) - the dimensions of the texture that will be rendered to.
    pub fn new(id: &str, dimensions: (u32, u32)) -> Self {
        Self {
            id: id.into(),
            dimensions,
            print_full_terminal: false,
            autoresize: false,
            autoresize_conversion_fn: None,
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

    /// Adds a bevy system that listens for terminal resize events and resizes the render texture
    /// to match the new dimensions.
    pub fn autoresize(mut self) -> Self {
        self.autoresize = true;
        self
    }

    /// Supply a function to customize how the render texture dimensions are calculated from the
    /// terminal dimensions. By default the ratio is 2-to-1, 2 pixels per character width and per
    /// character height.
    ///
    /// For example, if you are planning on displaying the bevy render on the left half of the
    /// terminal, keeping the right half free for other ratatui widgets, you could use the
    /// following function to resize the texture appropriately:
    ///
    /// ```no_run
    /// # use bevy::prelude::*;
    /// # use bevy_ratatui::RatatuiPlugins;
    /// # use bevy_ratatui_render::{RatatuiRenderContext, RatatuiRenderPlugin};
    /// #
    /// # fn main() {
    /// # App::new()
    /// #    .add_plugins((
    /// #        DefaultPlugins,
    /// #        RatatuiPlugins::default(),
    /// #
    /// RatatuiRenderPlugin::new("main", (0, 0))
    ///     .autoresize()
    ///     .autoresize_conversion_fn(|(width, height)| (width / 2, height)),
    /// #
    /// #    ));
    /// # }
    /// ```
    pub fn autoresize_conversion_fn(
        mut self,
        autoresize_conversion_fn: AutoresizeConversionFn,
    ) -> Self {
        self.autoresize_conversion_fn = Some(autoresize_conversion_fn);
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
            .world_mut()
            .get_resource_mut::<RatatuiRenderContext>()
            .is_none()
        {
            app.init_resource::<RatatuiRenderContext>()
                .add_systems(First, receive_rendered_images_system)
                .add_systems(PostUpdate, replaced_pipe_cleanup_system)
                .add_event::<ReplacedRenderPipeEvent>();

            let render_app = app.sub_app_mut(RenderApp);

            let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
            graph.add_node(ImageCopy, ImageCopyNode);
            graph.add_node_edge(bevy::render::graph::CameraDriverLabel, ImageCopy);

            render_app
                .add_systems(ExtractSchedule, image_copier_extract_system)
                .add_systems(Render, send_rendered_image_system.after(RenderSet::Render));
        }

        app.add_systems(
            PreStartup,
            initialize_context_system_generator(self.id.clone(), self.dimensions),
        );

        if self.print_full_terminal {
            app.add_systems(
                Update,
                print_full_terminal_system(self.id.clone()).map(error),
            );
        }

        if self.autoresize {
            app.add_systems(
                PostStartup,
                (
                    initial_resize_system,
                    autoresize_system_generator(self.id.clone(), self.autoresize_conversion_fn),
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                autoresize_system_generator(self.id.clone(), self.autoresize_conversion_fn),
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
    /// Create a render image for the given id. If an existing id is supplied, the existing render
    /// image is replaced.
    ///
    /// * `id` - Unique descriptive identifier, must match the id provided when the corresponding
    ///   `RatatuiRenderPlugin` was instantiated.
    ///
    /// * `dimensions` - New dimensions for the render image (`(width: u32, height: u32)`).
    ///
    /// * `world` - Mutable reference to Bevy world.
    pub fn create(id: &str, dimensions: (u32, u32), world: &mut World) {
        world.run_system_once(initialize_context_system_generator(id.into(), dimensions));
    }

    /// Gets a clone of the render target, for placement inside a bevy camera.
    ///
    /// * `id` - Unique descriptive identifier, must match the id provided when the corresponding
    ///   `RatatuiRenderPlugin` was instantiated.
    pub fn target(&self, id: &str) -> Option<RenderTarget> {
        let pipe = self.get(id)?;
        Some(pipe.target.clone())
    }

    /// Gets the dimensions of a given render image.
    ///
    /// * `id` - Unique descriptive identifier, must match the id provided when the corresponding
    ///   `RatatuiRenderPlugin` was instantiated.
    pub fn dimensions(&self, id: &str) -> Option<(u32, u32)> {
        let pipe = self.get(id)?;
        Some((pipe.image.width(), pipe.image.height()))
    }

    /// Gets a ratatui widget, that when drawn will print the most recent image rendered to the
    /// render target of the same id.
    ///
    /// * `id` - Unique descriptive identifier, must match the id provided when the corresponding
    ///   `RatatuiRenderPlugin` was instantiated.
    pub fn widget(&self, id: &str) -> Option<RatatuiRenderWidget> {
        let pipe = self.get(id)?;
        Some(RatatuiRenderWidget::new(&pipe.image))
    }
}

/// Creates a headless render pipe and adds it to the RatatuiRenderContext resource.
fn initialize_context_system_generator(
    id: String,
    dimensions: (u32, u32),
) -> impl FnMut(
    Commands,
    ResMut<Assets<Image>>,
    Res<RenderDevice>,
    ResMut<RatatuiRenderContext>,
    EventWriter<ReplacedRenderPipeEvent>,
) {
    move |mut commands, mut images, render_device, mut context, mut replaced_pipe| {
        let new_pipe =
            HeadlessRenderPipe::new(&mut commands, &mut images, &render_device, dimensions);
        let new_pipe_target = new_pipe.target.clone();
        let maybe_old_pipe = context.insert(id.clone(), new_pipe);

        if let Some(old_pipe) = maybe_old_pipe {
            replaced_pipe.send(ReplacedRenderPipeEvent {
                old_render_target: old_pipe.target,
                new_render_target: new_pipe_target,
            });
        }
    }
}

/// Draws the widget for the provided id in the full terminal, each frame.
fn print_full_terminal_system(
    id: String,
) -> impl FnMut(ResMut<RatatuiContext>, Res<RatatuiRenderContext>) -> io::Result<()> {
    move |mut ratatui, ratatui_render| {
        if let Some(render_widget) = ratatui_render.widget(&id) {
            ratatui.draw(|frame| {
                frame.render_widget(render_widget, frame.area());
            })?;
        }

        Ok(())
    }
}

/// Sends a single resize event during startup when autoresize is enabled.
fn initial_resize_system(
    ratatui: Res<RatatuiContext>,
    mut resize_events: EventWriter<ResizeEvent>,
) {
    if let Ok(size) = ratatui.size() {
        resize_events.send(ResizeEvent(size));
    }
}

/// Autoresizes the render texture to fit the terminal dimensions.
fn autoresize_system_generator(
    id: String,
    conversion_fn: Option<AutoresizeConversionFn>,
) -> impl FnMut(&mut World) {
    move |world| {
        let mut system_state: SystemState<EventReader<ResizeEvent>> = SystemState::new(world);
        let mut ratatui_events = system_state.get_mut(world);

        if let Some(ResizeEvent(dimensions)) = ratatui_events.read().last() {
            let terminal_dimensions = (dimensions.width as u32, dimensions.height as u32 * 2);
            let conversion_fn = conversion_fn.unwrap_or(|(width, height)| (width * 2, height * 2));
            let new_dimensions = conversion_fn(terminal_dimensions);
            RatatuiRenderContext::create(&id, new_dimensions, world);
        }
    }
}

#[derive(Event)]
pub struct ReplacedRenderPipeEvent {
    old_render_target: RenderTarget,
    new_render_target: RenderTarget,
}

/// When a new render pipe is created with an existing name, the old pipe is replaced.
/// This system cleans up assets and components from the old pipe.
fn replaced_pipe_cleanup_system(
    mut commands: Commands,
    mut replaced_pipe: EventReader<ReplacedRenderPipeEvent>,
    mut images: ResMut<Assets<Image>>,
    mut camera_query: Query<&mut Camera>,
    mut image_copier_query: Query<(Entity, &mut ImageCopier)>,
) {
    for ReplacedRenderPipeEvent {
        old_render_target,
        new_render_target,
    } in replaced_pipe.read()
    {
        if let Some(old_target_image) = old_render_target.as_image() {
            if let Some(mut camera) = camera_query.iter_mut().find(|camera| {
                if let Some(camera_image) = camera.target.as_image() {
                    return camera_image == old_target_image;
                }

                false
            }) {
                camera.target = new_render_target.clone();
                if let Some(image_handle) = old_render_target.as_image() {
                    images.remove(image_handle);
                }
            }

            if let Some((entity, image_copier)) = image_copier_query
                .iter_mut()
                .find(|(_, image_copier)| image_copier.src_image == *old_target_image)
            {
                commands.entity(entity).despawn();

                images.remove(&image_copier.src_image.clone());
            }
        };
    }
}
