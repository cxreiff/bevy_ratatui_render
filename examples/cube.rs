use std::io;
use std::time::Duration;

use bevy::app::AppExit;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::utils::error;
use bevy::window::ExitCondition;
use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_ratatui_render::{
    RatatuiContext, RatatuiEvent, RatatuiPlugin, RatatuiRenderContext, RatatuiRenderPlugin,
};
use crossterm::event;
use ratatui::layout::Alignment;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::widgets::Block;

#[derive(Component)]
pub struct Cube;

#[derive(Resource, Default)]
pub struct Flags {
    debug: bool,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    close_when_requested: false,
                }),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0)),
            FrameTimeDiagnosticsPlugin,
            RatatuiPlugin,
            RatatuiRenderPlugin::new(256, 256),
        ))
        .insert_resource(Flags::default())
        .insert_resource(InputState::Idle)
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, draw_scene.map(error))
        .add_systems(Update, handle_keys.map(error))
        .add_systems(Update, rotate_cube.after(handle_keys))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    rat_render: Res<RatatuiRenderContext>,
) {
    commands.spawn((
        Cube,
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(StandardMaterial {
                base_color: bevy::prelude::Color::rgb(100. / 256., 140. / 256., 180. / 256.),
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
        transform: Transform::from_xyz(3.0, 4.0, 6.0),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(3., 3., 3.0).looking_at(Vec3::ZERO, Vec3::Z),
        tonemapping: Tonemapping::None,
        camera: Camera {
            target: rat_render.target(),
            ..default()
        },
        ..default()
    });
}

fn draw_scene(
    mut rat: ResMut<RatatuiContext>,
    rat_render: Res<RatatuiRenderContext>,
    flags: Res<Flags>,
    diagnostics: Res<DiagnosticsStore>,
) -> io::Result<()> {
    let kitty_enabled = rat.kitty_enabled;
    rat.draw(|frame| {
        let mut block = Block::bordered()
            .bg(ratatui::style::Color::Rgb(0, 0, 0))
            .border_style(Style::default().bg(ratatui::style::Color::Rgb(0, 0, 0)));
        let inner = block.inner(frame.size());

        if flags.debug {
            block = block
                .title_top(format!(
                    "[kitty protocol: {}]",
                    if kitty_enabled { "enabled" } else { "disabled" }
                ))
                .title_alignment(Alignment::Right);

            if let Some(value) = diagnostics
                .get(&FrameTimeDiagnosticsPlugin::FPS)
                .and_then(|fps| fps.smoothed())
            {
                block = block
                    .title_top(format!("[fps: {value:.0}]"))
                    .title_alignment(Alignment::Right);
            }
        }

        frame.render_widget(block, frame.size());
        frame.render_widget(rat_render.widget(), inner);
    })?;

    Ok(())
}

#[derive(Resource)]
pub enum InputState {
    None,
    Idle,
    Left(f32),
    Right(f32),
}

pub fn handle_keys(
    mut rat_events: EventReader<RatatuiEvent>,
    mut exit: EventWriter<AppExit>,
    mut flags: ResMut<Flags>,
    mut input: ResMut<InputState>,
) -> io::Result<()> {
    for ev in rat_events.read() {
        if let RatatuiEvent(event::Event::Key(key_event)) = ev {
            match key_event.kind {
                event::KeyEventKind::Press | event::KeyEventKind::Repeat => match key_event.code {
                    event::KeyCode::Char('q') => {
                        exit.send(AppExit);
                    }

                    event::KeyCode::Char('d') => {
                        flags.debug = !flags.debug;
                    }

                    event::KeyCode::Left => {
                        *input = InputState::Left(0.75);
                    }

                    event::KeyCode::Right => {
                        *input = InputState::Right(0.75);
                    }

                    _ => {}
                },
                event::KeyEventKind::Release => match key_event.code {
                    event::KeyCode::Left => {
                        if let InputState::Left(_) = *input {
                            *input = InputState::None;
                        }
                    }
                    event::KeyCode::Right => {
                        if let InputState::Right(_) = *input {
                            *input = InputState::None;
                        }
                    }
                    _ => {}
                },
            }
        }
    }

    Ok(())
}

fn rotate_cube(
    time: Res<Time>,
    mut cube: Query<&mut Transform, With<Cube>>,
    mut input: ResMut<InputState>,
) {
    match *input {
        InputState::Idle => {
            cube.single_mut().rotate_z(time.delta_seconds());
        }
        InputState::Left(duration) => {
            cube.single_mut()
                .rotate_z(-time.delta_seconds() * duration.min(0.25) * 4.);
            let new_duration = (duration - time.delta_seconds()).max(0.);
            *input = if new_duration > 0. {
                InputState::Left(new_duration)
            } else {
                InputState::None
            }
        }
        InputState::Right(duration) => {
            cube.single_mut()
                .rotate_z(time.delta_seconds() * duration.min(0.25) * 4.);
            let new_duration = (duration - time.delta_seconds()).max(0.);
            *input = if new_duration > 0. {
                InputState::Right(new_duration)
            } else {
                InputState::None
            }
        }
        InputState::None => {}
    }
}
