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
use bevy_ratatui_render::LuminanceConfig;
use bevy_ratatui_render::RatatuiCamera;
use bevy_ratatui_render::RatatuiCameraPlugin;
use bevy_ratatui_render::RatatuiCameraStrategy;
use bevy_ratatui_render::RatatuiCameraWidget;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;

mod shared;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
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
        RatatuiCameraStrategy::Luminance(LuminanceConfig::default()),
        Camera3d::default(),
        Transform::from_xyz(0., 3., 0.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    commands.spawn((
        RatatuiCamera::default(),
        Camera3d::default(),
        Transform::from_xyz(0., 0., 3.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    commands.spawn((
        RatatuiCamera::default(),
        RatatuiCameraStrategy::Luminance(LuminanceConfig::default()),
        Camera3d::default(),
        Transform::from_xyz(2., 2., 2.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}

pub fn draw_scene_system(
    mut ratatui: ResMut<RatatuiContext>,
    ratatui_camera_widgets: Query<&RatatuiCameraWidget>,
    flags: Res<shared::Flags>,
    diagnostics: Res<DiagnosticsStore>,
    kitty_enabled: Option<Res<KittyEnabled>>,
) -> std::io::Result<()> {
    ratatui.draw(|frame| {
        let area = shared::debug_frame(frame, &flags, &diagnostics, kitty_enabled.as_deref());

        let widgets = ratatui_camera_widgets
            .iter()
            .enumerate()
            .collect::<Vec<_>>();

        let layout = Layout::new(
            Direction::Horizontal,
            vec![Constraint::Fill(1); widgets.len()],
        )
        .split(area);

        for (i, widget) in widgets {
            frame.render_widget(widget, layout[i]);
        }
    })?;

    Ok(())
}
