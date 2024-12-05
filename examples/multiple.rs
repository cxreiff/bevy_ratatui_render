use std::io;
use std::time::Duration;

use bevy::app::AppExit;
use bevy::app::ScheduleRunnerPlugin;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::utils::error;
use bevy::winit::WinitPlugin;
use bevy_ratatui::event::KeyEvent;
use bevy_ratatui::kitty::KittyEnabled;
use bevy_ratatui::terminal::RatatuiContext;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_render::{RatatuiRenderContext, RatatuiRenderPlugin};
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Padding};

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
                .disable::<WinitPlugin>()
                .disable::<LogPlugin>(),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 60.)),
            FrameTimeDiagnosticsPlugin,
            RatatuiPlugins::default(),
            RatatuiRenderPlugin::new("top_left", (128, 128)),
            RatatuiRenderPlugin::new("top_right", (128, 128)),
            RatatuiRenderPlugin::new("bottom", (256, 128)),
        ))
        .insert_resource(Flags::default())
        .insert_resource(InputState::Idle)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup_scene_system)
        .add_systems(Update, draw_scene_system.map(error))
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
        PointLight {
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(3., 4., 6.),
    ));
    commands.spawn((
        Camera3d::default(),
        Camera {
            target: ratatui_render.target("top_left").unwrap(),
            ..default()
        },
        Transform::from_xyz(0., 3., 0.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    commands.spawn((
        Camera3d::default(),
        Camera {
            target: ratatui_render.target("top_right").unwrap(),
            ..default()
        },
        Transform::from_xyz(0., 0., 3.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    commands.spawn((
        Camera3d::default(),
        Camera {
            target: ratatui_render.target("bottom").unwrap(),
            ..default()
        },
        Transform::from_xyz(2., 2., 2.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}

fn draw_scene_system(
    mut ratatui: ResMut<RatatuiContext>,
    ratatui_render: Res<RatatuiRenderContext>,
    flags: Res<Flags>,
    diagnostics: Res<DiagnosticsStore>,
    kitty_enabled: Option<Res<KittyEnabled>>,
) -> io::Result<()> {
    ratatui.draw(|frame| {
        let mut block = Block::bordered()
            .bg(ratatui::style::Color::Rgb(0, 0, 0))
            .border_style(Style::default().bg(ratatui::style::Color::Rgb(0, 0, 0)));

        let bottom_block = block.clone();
        let top_left_block = block.clone();
        let top_right_block = block.clone();

        block = block
            .padding(Padding::proportional(1))
            .title_bottom("[q for quit]")
            .title_bottom("[d for debug]")
            .title_alignment(Alignment::Center);

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

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(block.inner(frame.area()));

        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(50),
                Constraint::Length(1),
                Constraint::Percentage(50),
            ])
            .split(layout[0]);

        let top_left = top_layout[0];
        let top_right = top_layout[2];
        let bottom = layout[1];

        let inner_top_left = top_left_block.inner(top_left);
        let inner_top_right = top_right_block.inner(top_right);
        let inner_bottom = bottom_block.inner(bottom);

        let top_left_widget = ratatui_render.widget("top_left").unwrap();
        let top_right_widget = ratatui_render.widget("top_right").unwrap();
        let bottom_widget = ratatui_render.widget("bottom").unwrap();

        frame.render_widget(block, frame.area());
        frame.render_widget(top_left_block, top_left);
        frame.render_widget(bottom_block, top_right);
        frame.render_widget(top_right_block, bottom);
        frame.render_widget(top_left_widget, inner_top_left);
        frame.render_widget(top_right_widget, inner_top_right);
        frame.render_widget(bottom_widget, inner_bottom);
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
    mut ratatui_events: EventReader<KeyEvent>,
    mut exit: EventWriter<AppExit>,
    mut flags: ResMut<Flags>,
    mut input: ResMut<InputState>,
) {
    for key_event in ratatui_events.read() {
        match key_event.kind {
            KeyEventKind::Press | KeyEventKind::Repeat => match key_event.code {
                KeyCode::Char('q') => {
                    exit.send_default();
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
            cube.single_mut().rotate_z(time.delta_secs());
        }
        InputState::Left(duration) => {
            cube.single_mut()
                .rotate_z(-time.delta_secs() * duration.min(0.25) * 4.);
            let new_duration = (duration - time.delta_secs()).max(0.);
            *input = if new_duration > 0. {
                InputState::Left(new_duration)
            } else {
                InputState::None
            }
        }
        InputState::Right(duration) => {
            cube.single_mut()
                .rotate_z(time.delta_secs() * duration.min(0.25) * 4.);
            let new_duration = (duration - time.delta_secs()).max(0.);
            *input = if new_duration > 0. {
                InputState::Right(new_duration)
            } else {
                InputState::None
            }
        }
        InputState::None => {}
    }
}
