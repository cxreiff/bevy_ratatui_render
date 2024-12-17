use std::io;
use std::time::Duration;

use bevy::app::AppExit;
use bevy::app::ScheduleRunnerPlugin;
use bevy::color::Color;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::utils::error;
use bevy::winit::WinitPlugin;
use bevy_ratatui::event::KeyEvent;
use bevy_ratatui::kitty::KittyEnabled;
use bevy_ratatui::terminal::RatatuiContext;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_render::LuminanceConfig;
use bevy_ratatui_render::RatatuiCamera;
use bevy_ratatui_render::RatatuiCameraEdgeDetection;
use bevy_ratatui_render::RatatuiCameraPlugin;
use bevy_ratatui_render::RatatuiCameraStrategy;
use bevy_ratatui_render::RatatuiCameraWidget;
use crossterm::event::{KeyCode, KeyEventKind};
use log::LevelFilter;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::widgets::Block;
use tui_logger::init_logger;
use tui_logger::set_default_level;
use tui_logger::TuiLoggerWidget;

#[derive(Component)]
pub struct Cube;

#[derive(Resource, Default)]
pub struct Flags {
    debug: bool,
}

fn main() {
    init_logger(LevelFilter::Info).unwrap();
    set_default_level(LevelFilter::Info);

    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
                .disable::<WinitPlugin>()
                .disable::<LogPlugin>(),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 90.)),
            FrameTimeDiagnosticsPlugin,
            RatatuiPlugins::default(),
            RatatuiCameraPlugin,
        ))
        .insert_resource(Flags { debug: false })
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup_scene_system)
        .add_systems(Update, draw_scene_system.map(error))
        .add_systems(Update, rotate_cube_system)
        .add_systems(PreUpdate, handle_input_system)
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
            base_color: Color::srgb(0.6, 0.7, 0.9),
            ..Default::default()
        })),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(15., 15., 1.))),
        Transform::from_xyz(0., 0., -6.),
    ));
    commands.spawn((PointLight::default(), Transform::from_xyz(3., 4., 6.)));
    commands.spawn((
        RatatuiCamera {
            strategy: RatatuiCameraStrategy::Luminance(LuminanceConfig::default()),
            autoresize: true,
            ..default()
        },
        RatatuiCameraEdgeDetection::default(),
        Camera3d::default(),
        Transform::from_xyz(2.5, 2.5, 2.5).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}

fn draw_scene_system(
    mut ratatui: ResMut<RatatuiContext>,
    ratatui_camera_widget: Query<&RatatuiCameraWidget>,
    flags: Res<Flags>,
    diagnostics: Res<DiagnosticsStore>,
    kitty_enabled: Option<Res<KittyEnabled>>,
) -> io::Result<()> {
    ratatui.draw(|frame| {
        let layout = Layout::new(
            Direction::Vertical,
            [Constraint::Percentage(66), Constraint::Fill(1)],
        )
        .split(frame.area());

        let mut block = Block::bordered()
            .bg(ratatui::style::Color::Rgb(0, 0, 0))
            .border_style(Style::default().bg(ratatui::style::Color::Rgb(0, 0, 0)))
            .title_bottom("[q for quit]")
            .title_bottom("[d for debug]")
            .title_bottom("[p for panic]")
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

        if flags.debug {
            let inner = block.inner(layout[0]);
            frame.render_widget(block, layout[0]);
            frame.render_widget(
                TuiLoggerWidget::default()
                    .block(Block::bordered())
                    .style(Style::default().bg(ratatui::style::Color::Reset)),
                layout[1],
            );
            if let Ok(camera_widget) = ratatui_camera_widget.get_single() {
                frame.render_widget(camera_widget, inner);
            }
        } else {
            let inner = block.inner(frame.area());
            frame.render_widget(block, frame.area());
            if let Ok(camera_widget) = ratatui_camera_widget.get_single() {
                frame.render_widget(camera_widget, inner);
            }
        }
    })?;

    Ok(())
}

pub fn handle_input_system(
    mut rat_events: EventReader<KeyEvent>,
    mut exit: EventWriter<AppExit>,
    mut flags: ResMut<Flags>,
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
                _ => {}
            },
            _ => {}
        }
    }
}

fn rotate_cube_system(time: Res<Time>, mut cube: Query<&mut Transform, With<Cube>>) {
    cube.single_mut().rotate_z(time.delta_secs());
}
