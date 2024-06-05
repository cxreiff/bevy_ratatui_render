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
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
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
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 60.)),
            FrameTimeDiagnosticsPlugin,
            RatatuiPlugin,
            RatatuiRenderPlugin::new()
                .add_render((128, 128))
                .add_render((128, 128))
                .add_render((256, 128)),
        ))
        .insert_resource(Flags::default())
        .insert_resource(InputState::Idle)
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_systems(Startup, setup_scene_system)
        .add_systems(Update, draw_scene_system.map(error))
        .add_systems(Update, handle_input_system.map(error))
        .add_systems(Update, rotate_cube_system.after(handle_input_system))
        .run();
}

fn setup_scene_system(
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
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(3., 4., 6.),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 3., 0.).looking_at(Vec3::ZERO, Vec3::Z),
        tonemapping: Tonemapping::None,
        camera: Camera {
            target: rat_render.target(0),
            ..default()
        },
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 0., 3.).looking_at(Vec3::ZERO, Vec3::Z),
        tonemapping: Tonemapping::None,
        camera: Camera {
            target: rat_render.target(1),
            ..default()
        },
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(2., 2., 2.).looking_at(Vec3::ZERO, Vec3::Z),
        tonemapping: Tonemapping::None,
        camera: Camera {
            target: rat_render.target(2),
            ..default()
        },
        ..default()
    });
}

fn draw_scene_system(
    mut rat: ResMut<RatatuiContext>,
    rat_render: Res<RatatuiRenderContext>,
    flags: Res<Flags>,
    diagnostics: Res<DiagnosticsStore>,
) -> io::Result<()> {
    let kitty_enabled = rat.kitty_enabled;
    rat.draw(|frame| {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.size());

        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout[0]);

        let mut block = Block::bordered()
            .bg(ratatui::style::Color::Rgb(0, 0, 0))
            .border_style(Style::default().bg(ratatui::style::Color::Rgb(0, 0, 0)));

        let inner_top_left = block.inner(top_layout[0]);
        let inner_top_right = block.inner(top_layout[1]);
        let inner_bottom = block.inner(layout[1]);

        frame.render_widget(block.clone(), top_layout[0]);
        frame.render_widget(block.clone(), layout[1]);

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

        frame.render_widget(block, top_layout[1]);
        frame.render_widget(rat_render.widget(0), inner_top_left);
        frame.render_widget(rat_render.widget(1), inner_top_right);
        frame.render_widget(rat_render.widget(2), inner_bottom);
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

pub fn handle_input_system(
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

fn rotate_cube_system(
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
