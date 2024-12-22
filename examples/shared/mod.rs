use bevy::app::AppExit;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use bevy_ratatui::kitty::KittyEnabled;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::widgets::Block;
use ratatui::Frame;
use tui_logger::TuiLoggerWidget;

#[allow(dead_code)]
#[derive(Component)]
pub struct Spinner;

#[allow(dead_code)]
#[derive(Resource, Default)]
pub struct Flags {
    pub debug: bool,
}

#[allow(dead_code)]
pub fn spawn_3d_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    commands.spawn((
        Spinner,
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
}

#[allow(dead_code)]
pub fn spawn_2d_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    commands.spawn((
        Spinner,
        Mesh2d(meshes.add(RegularPolygon::new(66.0, 8))),
        MeshMaterial2d(materials.add(Color::srgb(0.4, 0.4, 0.6))),
    ));
}

#[allow(dead_code)]
pub fn debug_frame(
    frame: &mut Frame,
    flags: &Flags,
    diagnostics: &DiagnosticsStore,
    kitty_enabled: Option<&KittyEnabled>,
) -> Rect {
    let mut block = Block::bordered()
        .bg(ratatui::style::Color::Rgb(0, 0, 0))
        .border_style(Style::default().bg(ratatui::style::Color::Rgb(0, 0, 0)))
        .title_bottom("[q for quit]")
        .title_bottom("[d for debug]")
        .title_bottom("[p for panic]")
        .title_alignment(Alignment::Center);

    if flags.debug {
        let layout = Layout::new(
            Direction::Vertical,
            [Constraint::Percentage(66), Constraint::Fill(1)],
        )
        .split(frame.area());

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

        let inner = block.inner(layout[0]);
        frame.render_widget(block, layout[0]);
        frame.render_widget(
            TuiLoggerWidget::default()
                .block(Block::bordered())
                .style(Style::default().bg(ratatui::style::Color::Reset)),
            layout[1],
        );

        inner
    } else {
        let inner = block.inner(frame.area());
        frame.render_widget(block, frame.area());

        inner
    }
}

#[allow(dead_code)]
#[derive(Resource, Default)]
pub enum InputState {
    None,
    #[default]
    Idle,
    Left(f32),
    Right(f32),
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn rotate_spinners_system(
    time: Res<Time>,
    mut cube: Query<&mut Transform, With<Spinner>>,
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
