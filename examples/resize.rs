use std::time::Duration;

use bevy::app::AppExit;
use bevy::app::ScheduleRunnerPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy_ratatui::event::KeyEvent;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_render::RatatuiCamera;
use bevy_ratatui_render::RatatuiCameraPlugin;
use crossterm::event::{KeyCode, KeyEventKind};

#[derive(Component)]
pub struct Cube;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .disable::<WinitPlugin>()
                .disable::<LogPlugin>(),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 60.)),
            RatatuiPlugins::default(),
            RatatuiCameraPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup_scene_system)
        .add_systems(Update, handle_input_system)
        .add_systems(Update, rotate_cube_system.after(handle_input_system))
        .run();
}

fn setup_scene_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Cube,
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.54, 0.7),
            ..Default::default()
        })),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(15., 15., 1.))),
        Transform::from_xyz(0., 0., -6.),
    ));
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(3., 4., 6.),
    ));
    commands.spawn((
        RatatuiCamera {
            autoprint: true,
            autoresize: true,
            autoresize_function: |(width, height)| (width * 4, height * 3),
            ..default()
        },
        Camera3d::default(),
        Transform::from_xyz(3., 3., 3.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}

pub fn handle_input_system(
    mut ratatui_events: EventReader<KeyEvent>,
    mut exit: EventWriter<AppExit>,
) {
    for key_event in ratatui_events.read() {
        if let KeyEventKind::Press | KeyEventKind::Repeat = key_event.kind {
            if let KeyCode::Char('q') = key_event.code {
                exit.send_default();
            }
        }
    }
}

fn rotate_cube_system(time: Res<Time>, mut cube: Query<&mut Transform, With<Cube>>) {
    cube.single_mut().rotate_z(time.delta_secs());
}
