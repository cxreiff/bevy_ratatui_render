use bevy::utils::error;
use bevy::{app::AppExit, prelude::*};
use crossterm::event;
use ratatui::prelude::*;
use std::io::{self, Stdout};

pub mod ascii;
pub mod headless;
mod tui;

pub struct RatatuiPlugin;

impl Plugin for RatatuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RatatuiEvent>()
            .add_systems(Startup, ratatui_startup.map(error))
            .add_systems(Update, ratatui_input.map(error))
            .add_systems(Last, ratatui_cleanup.map(error));
    }
}

#[derive(Resource)]
pub struct RatatuiResource {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

fn ratatui_startup(mut commands: Commands) -> io::Result<()> {
    tui::init_panic_hooks();
    let mut terminal = tui::init()?;
    terminal.clear()?;
    commands.insert_resource(RatatuiResource { terminal });

    Ok(())
}

fn ratatui_cleanup(mut events: EventReader<AppExit>) -> io::Result<()> {
    for _ in events.read() {
        tui::restore()?;
    }

    Ok(())
}

#[derive(Event)]
pub struct RatatuiEvent(pub event::Event);

fn ratatui_input(mut event_writer: EventWriter<RatatuiEvent>) -> io::Result<()> {
    if event::poll(std::time::Duration::from_millis(16))? {
        event_writer.send(RatatuiEvent(event::read()?));
    }
    Ok(())
}
