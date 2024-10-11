use std::io;
use std::time::Duration;

use bevy::app::AppExit;
use bevy::app::ScheduleRunnerPlugin;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::utils::error;
use bevy::winit::WinitPlugin;
use bevy_ratatui::event::KeyEvent;
use bevy_ratatui::kitty::KittyEnabled;
use bevy_ratatui::terminal::RatatuiContext;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_render::{RatatuiRenderContext, RatatuiRenderPlugin};
use crossterm::event::{KeyCode, KeyEventKind};
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
            DefaultPlugins.build().disable::<WinitPlugin>(),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 60.)),
            FrameTimeDiagnosticsPlugin,
            RatatuiPlugins::default(),
            RatatuiRenderPlugin::new("main", (256, 256)).autoresize(),
        ))
        .insert_resource(Flags::default())
        .insert_resource(InputState::Idle)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup_scene_system)
        .add_systems(Update, draw_scene_system.map(error))
        .add_systems(PreUpdate, handle_input_system)
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

fn draw_scene_system(
    mut ratatui: ResMut<RatatuiContext>,
    rat_render: Res<RatatuiRenderContext>,
    flags: Res<Flags>,
    diagnostics: Res<DiagnosticsStore>,
    kitty_enabled: Option<Res<KittyEnabled>>,
) -> io::Result<()> {
    ratatui.draw(|frame| {
        let mut block = Block::bordered()
            .bg(ratatui::style::Color::Rgb(0, 0, 0))
            .border_style(Style::default().bg(ratatui::style::Color::Rgb(0, 0, 0)))
            .title_bottom("[q for quit]")
            .title_bottom("[d for debug]")
            .title_bottom("[p for panic]")
            .title_alignment(Alignment::Center);

        let inner = block.inner(frame.area());

        if flags.debug {
            block = block.title_top(format!(
                "[kitty protocol: {}]",
                if kitty_enabled.is_some() {
                    "enabled"
                } else {
                    "disabled"
                }
            ));

            if let Some(value) = diagnostics
                .get(&FrameTimeDiagnosticsPlugin::FPS)
                .and_then(|fps| fps.smoothed())
            {
                block = block.title_top(format!("[fps: {value:.0}]"));
            }
        }

        frame.render_widget(block, frame.area());
        frame.render_widget(rat_render.widget("main").unwrap(), inner);
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
    mut rat_events: EventReader<KeyEvent>,
    mut exit: EventWriter<AppExit>,
    mut flags: ResMut<Flags>,
    mut input: ResMut<InputState>,
) {
    for key_event in rat_events.read() {
        match key_event.kind {
            KeyEventKind::Press | KeyEventKind::Repeat => match key_event.code {
                KeyCode::Char('q') => {
                    exit.send_default();
                }

                KeyCode::Char('p') => {
                    panic!("Panic!");
                }

                KeyCode::Char('d') => {
                    flags.debug = !flags.debug;
                }

                KeyCode::Left => {
                    *input = InputState::Left(0.75);
                }

                KeyCode::Right => {
                    *input = InputState::Right(0.75);
                }

                _ => {}
            },
            KeyEventKind::Release => match key_event.code {
                KeyCode::Left => {
                    if let InputState::Left(_) = *input {
                        *input = InputState::None;
                    }
                }
                KeyCode::Right => {
                    if let InputState::Right(_) = *input {
                        *input = InputState::None;
                    }
                }
                _ => {}
            },
        }
    }
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
