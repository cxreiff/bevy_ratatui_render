use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::color::Color;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::utils::error;
use bevy::winit::WinitPlugin;
use bevy_ratatui::kitty::KittyEnabled;
use bevy_ratatui::terminal::RatatuiContext;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_render::LuminanceConfig;
use bevy_ratatui_render::RatatuiCamera;
use bevy_ratatui_render::RatatuiCameraPlugin;
use bevy_ratatui_render::RatatuiCameraStrategy;
use bevy_ratatui_render::RatatuiCameraWidget;
use log::LevelFilter;
use tui_logger::init_logger;
use tui_logger::set_default_level;

mod shared;

fn main() {
    init_logger(LevelFilter::Info).unwrap();
    set_default_level(LevelFilter::Info);

    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
                .disable::<WinitPlugin>()
                .disable::<LogPlugin>(),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 90.)),
            FrameTimeDiagnosticsPlugin,
            RatatuiPlugins::default(),
            RatatuiCameraPlugin,
        ))
        .init_resource::<shared::Flags>()
        .init_resource::<shared::InputState>()
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup_scene_system)
        .add_systems(Update, draw_scene_system.map(error))
        .add_systems(Update, shared::rotate_spinners_system)
        .add_systems(PreUpdate, shared::handle_input_system)
        .run();
}

fn setup_scene_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    shared::spawn_2d_scene(&mut commands, &mut meshes, &mut materials);

    commands.spawn((
        RatatuiCamera::autoresize(),
        RatatuiCameraStrategy::Luminance(LuminanceConfig::default()),
        Camera2d,
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
