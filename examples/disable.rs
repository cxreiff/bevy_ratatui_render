use std::io;

use bevy::app::AppExit;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::utils::error;
use bevy_ratatui::event::KeyEvent;
use bevy_ratatui::terminal::RatatuiContext;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_render::{RatatuiRenderContext, RatatuiRenderPlugin};
use crossterm::event::{KeyCode, KeyEventKind, KeyEventState, KeyModifiers};

// Uncomment these imports, `ScheduleRunnerPlugin`, and remove `.disable()` from
// `RatatuiRenderPlugin` in order to return to terminal rendering mode.
// use bevy::app::ScheduleRunnerPlugin;
// use std::time::Duration;
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 60.)),
            FrameTimeDiagnosticsPlugin,
            RatatuiPlugins::default(),
            RatatuiRenderPlugin::new("main", (256, 256)).disable(),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup_camera_system)
        .add_systems(Startup, setup_scene_system)
        .add_systems(Update, draw_scene_system.map(error))
        .add_systems(
            Update,
            passthrough_keyboard_events_system.before(handle_input_system),
        )
        .add_systems(Update, handle_input_system)
        .run();
}

// Use `unwrap_or_default` so the camera falls back to a normal window target.
fn setup_camera_system(mut commands: Commands, ratatui_render: Res<RatatuiRenderContext>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(3., 3., 3.).looking_at(Vec3::ZERO, Vec3::Z),
        tonemapping: Tonemapping::None,
        camera: Camera {
            target: ratatui_render.target("main").unwrap_or_default(),
            ..default()
        },
        ..default()
    });
}

// Wrap `frame.render_widget` in an if-let for when `widget(id)` returns `None`.
fn draw_scene_system(
    mut ratatui: ResMut<RatatuiContext>,
    rat_render: Res<RatatuiRenderContext>,
) -> io::Result<()> {
    ratatui.draw(|frame| {
        if let Some(widget) = rat_render.widget("main") {
            frame.render_widget(widget, frame.area());
        }
    })?;

    Ok(())
}

// Listen for normal bevy input and send equivalent terminal events to get picked up by your systems.
fn passthrough_keyboard_events_system(
    mut read_keyboard: EventReader<KeyboardInput>,
    mut write_crossterm: EventWriter<KeyEvent>,
) {
    for ev in read_keyboard.read() {
        write_crossterm.send(KeyEvent(crossterm::event::KeyEvent {
            code: match ev.key_code {
                bevy::prelude::KeyCode::ArrowLeft => KeyCode::Left,
                bevy::prelude::KeyCode::ArrowRight => KeyCode::Right,
                bevy::prelude::KeyCode::KeyQ => KeyCode::Char('q'),
                _ => KeyCode::Null,
            },
            kind: match ev.state {
                ButtonState::Pressed => KeyEventKind::Press,
                ButtonState::Released => KeyEventKind::Release,
            },
            state: KeyEventState::NONE,
            modifiers: KeyModifiers::NONE,
        }));
    }
}

#[derive(Component)]
pub struct Cube;

fn setup_scene_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(3., 4., 6.),
        ..default()
    });
}

pub fn handle_input_system(
    mut rat_events: EventReader<KeyEvent>,
    mut cube: Query<&mut Transform, With<Cube>>,
    mut exit: EventWriter<AppExit>,
    time: Res<Time>,
) {
    for key_event in rat_events.read() {
        match key_event.kind {
            KeyEventKind::Press | KeyEventKind::Repeat => match key_event.code {
                KeyCode::Char('q') => {
                    exit.send_default();
                }

                KeyCode::Left => {
                    cube.single_mut().rotate_z(-10. * time.delta_seconds());
                }

                KeyCode::Right => {
                    cube.single_mut().rotate_z(10. * time.delta_seconds());
                }

                _ => {}
            },
            _ => {}
        }
    }
}
