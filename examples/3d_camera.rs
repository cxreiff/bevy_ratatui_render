use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::utils::error;
use bevy::winit::WinitPlugin;
use bevy_ratatui::kitty::KittyEnabled;
use bevy_ratatui::terminal::RatatuiContext;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_render::RatatuiCamera;
use bevy_ratatui_render::RatatuiCameraPlugin;
use bevy_ratatui_render::RatatuiCameraWidget;

mod shared;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
                .disable::<WinitPlugin>()
                .disable::<LogPlugin>(),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 60.)),
            FrameTimeDiagnosticsPlugin,
            RatatuiPlugins::default(),
            RatatuiCameraPlugin,
        ))
        .init_resource::<shared::Flags>()
        .init_resource::<shared::InputState>()
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup_scene_system)
        .add_systems(Update, draw_scene_system.map(error))
        .add_systems(PreUpdate, shared::handle_input_system)
        .add_systems(Update, shared::rotate_spinners_system)
        .run();
}

fn setup_scene_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    shared::spawn_3d_scene(&mut commands, &mut meshes, &mut materials);

    commands.spawn((
        RatatuiCamera::default(),
        Camera3d::default(),
        Transform::from_xyz(3., 3., 3.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}

pub fn draw_scene_system(
    mut ratatui: ResMut<RatatuiContext>,
    ratatui_camera_widget: Query<&RatatuiCameraWidget>,
    flags: Res<shared::Flags>,
    diagnostics: Res<DiagnosticsStore>,
    kitty_enabled: Option<Res<KittyEnabled>>,
) -> std::io::Result<()> {
    ratatui.draw(|frame| {
        let area = shared::debug_frame(frame, &flags, &diagnostics, kitty_enabled.as_deref());

        if let Ok(camera_widget) = ratatui_camera_widget.get_single() {
            frame.render_widget(camera_widget, area);
        }
    })?;

    Ok(())
}
