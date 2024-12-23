use bevy::prelude::*;

use crate::{
    camera_node::RatatuiCameraNodePlugin, camera_node_sobel::RatatuiCameraNodeSobelPlugin,
    camera_readback::RatatuiCameraReadbackPlugin,
};

/// Add this plugin, add a RatatuiCamera component to your camera, and then a RatatuiCameraWidget
/// component will be made available in your camera entity. Use the RatatuiContext provided by
/// bevy_ratatui and this widget to draw the camera's rendered output to the terminal.
///
/// # Example:
///
/// ```no_run
/// # use std::time::Duration;
/// # use bevy::app::ScheduleRunnerPlugin;
/// # use bevy::winit::WinitPlugin;
/// # use bevy::prelude::*;
/// # use bevy::utils::error;
/// # use bevy::log::LogPlugin;
/// # use bevy_ratatui::RatatuiPlugins;
/// # use bevy_ratatui::terminal::RatatuiContext;
/// # use bevy_ratatui_render::{RatatuiCamera, RatatuiCameraPlugin, RatatuiCameraWidget};
/// #
/// fn main() {
///     App::new()
///         .add_plugins((
///             // disable WinitPlugin as it panics in environments without a display server.
///             // disable LogPlugin as it interferes with terminal output.
///             DefaultPlugins.build()
///                 .disable::<WinitPlugin>()
///                 .disable::<LogPlugin>(),
///
///             // create windowless loop and set its frame rate.
///             ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 60.)),
///
///             // set up the Ratatui context and forward terminal input events.
///             RatatuiPlugins::default(),
///
///             // add the ratatui camera plugin.
///             RatatuiCameraPlugin,
///         ))
///         .add_systems(Startup, setup_scene_system)
///         .add_systems(PostUpdate, draw_scene_system.map(error));
/// }
///
/// // add RatatuiCamera to your scene's camera.
/// fn setup_scene_system(mut commands: Commands) {
///     commands.spawn((
///         Camera3d::default(),
///         RatatuiCamera::default(),
///     ));
/// }
///
/// // a RatatuiCameraWidget component will be available in your camera entity.
/// fn draw_scene_system(
///     mut ratatui: ResMut<RatatuiContext>,
///     camera_widget: Query<&RatatuiCameraWidget>,
/// ) -> std::io::Result<()> {
///     ratatui.draw(|frame| {
///         frame.render_widget(camera_widget.single(), frame.area());
///     })?;
///
///     Ok(())
/// }
/// ```
///
pub struct RatatuiCameraPlugin;

impl Plugin for RatatuiCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RatatuiCameraNodePlugin,
            RatatuiCameraNodeSobelPlugin,
            RatatuiCameraReadbackPlugin,
        ));
    }
}
