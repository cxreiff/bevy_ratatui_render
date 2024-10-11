use std::time::Duration;

use bevy::app::AppExit;
use bevy::app::ScheduleRunnerPlugin;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::winit::WinitPlugin;
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
                .disable::<WinitPlugin>(),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 60.)),
            RatatuiPlugins::default(),
            RatatuiRenderPlugin::new("main", (256, 256))
                .print_full_terminal()
                .autoresize()
                .autoresize_conversion_fn(|(width, height)| (width * 4, height * 3)),
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
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.4, 0.54, 0.7),
                ..Default::default()
            }),
            transform: Transform::default(),
            ..Default::default()
        },
    ));
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(15., 15., 1.)),
        material: materials.add(StandardMaterial::default()),
        transform: Transform::from_xyz(0., 0., -6.),
        ..Default::default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(3., 4., 6.),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(3., 3., 3.).looking_at(Vec3::ZERO, Vec3::Z),
        tonemapping: Tonemapping::None,
        camera: Camera {
            target: ratatui_render.target("main").unwrap(),
            ..default()
        },
        ..default()
    });
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
    cube.single_mut().rotate_z(time.delta_seconds());
}
