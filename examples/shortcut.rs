use std::time::Duration;

use bevy::app::AppExit;
use bevy::log::LogPlugin;
use bevy::winit::WinitPlugin;
use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_ratatui::event::KeyEvent;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_render::{RatatuiRenderContext, RatatuiRenderPlugin};
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
            RatatuiRenderPlugin::new("main", (256, 256))
                .print_full_terminal()
                .autoresize(),
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
    ratatui_render: Res<RatatuiRenderContext>,
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
        Camera3d::default(),
        Camera {
            target: ratatui_render.target("main").unwrap(),
            ..default()
        },
        Transform::from_xyz(3., 3., 3.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}

pub fn handle_input_system(mut rat_events: EventReader<KeyEvent>, mut exit: EventWriter<AppExit>) {
    for key_event in rat_events.read() {
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
