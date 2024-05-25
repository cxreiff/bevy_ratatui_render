use std::io;
use std::time::Duration;

use bevy::app::AppExit;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::utils::error;
use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_rat::ascii::AsciiPlugin;
use bevy_rat::headless::receive_render_image;
use bevy_rat::RatatuiEvent;
use bevy_rat::{RatatuiPlugin, RatatuiResource};
use crossterm::event;
use image::{imageops, DynamicImage, GenericImageView};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Stylize;
use ratatui::style::{Color, Style};
use ratatui::widgets::Block;
use ratatui_image::{
    picker::{Picker, ProtocolType},
    Image, Resize,
};

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
                    exit_condition: bevy::window::ExitCondition::DontExit,
                    close_when_requested: false,
                }),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 30.0)),
            FrameTimeDiagnosticsPlugin,
            AsciiPlugin,
            RatatuiPlugin,
        ))
        .insert_resource(Flags::default())
        .insert_resource(InputState::Idle)
        .add_systems(Startup, setup)
        .add_systems(Update, receive_render_image.pipe(ratatui_render).map(error))
        .add_systems(Update, handle_keys.map(error))
        .add_systems(Update, rotate_cube.after(handle_keys))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Cube,
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(StandardMaterial::default()),
            transform: Transform::default(),
            ..Default::default()
        },
    ));
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

fn ratatui_render(
    In(image): In<Option<DynamicImage>>,
    mut rat: ResMut<RatatuiResource>,
    flags: Res<Flags>,
    diagnostics: Res<DiagnosticsStore>,
) -> io::Result<()> {
    if let Some(image) = image {
        let mut picker = Picker::new((1, 2));
        picker.protocol_type = ProtocolType::Halfblocks;

        rat.terminal.draw(|frame| {
            let mut block = Block::bordered()
                .bg(Color::Rgb(0, 0, 0))
                .border_style(Style::default().bg(Color::Rgb(0, 0, 0)));
            let mut inner = block.inner(frame.size());

            let Rect { width, height, .. } = frame.size();
            let image = image.resize(
                width as u32,
                height as u32 * 2,
                imageops::FilterType::Nearest,
            );

            let (image_width, image_height) = image.dimensions();
            inner.x += inner.width.saturating_sub(image_width as u16) / 2;
            inner.y += (inner.height * 2).saturating_sub(image_height as u16) / 4;

            let img_as_blocks = picker
                .new_protocol(image, frame.size(), Resize::Fit(None))
                .unwrap();

            if flags.debug {
                if let Some(value) = diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(|fps| fps.smoothed())
                {
                    block = block
                        .title_top(format!("{value:.0}"))
                        .title_alignment(Alignment::Right);
                }
            }

            frame.render_widget(block, frame.size());

            frame.render_widget(Image::new(img_as_blocks.as_ref()), inner);
        })?;
    }

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
            if key_event.kind == event::KeyEventKind::Press {
                match key_event.code {
                    event::KeyCode::Char('q') => {
                        exit.send(AppExit::Success);
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
                };
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
