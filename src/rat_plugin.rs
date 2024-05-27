use std::io::{self, Stdout};

use bevy::{prelude::*, utils::error};
use crossterm::event;
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::rat_tui;

pub struct RatPlugin;

impl Plugin for RatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RatEvent>()
            .add_systems(Startup, ratatui_startup.map(error))
            .add_systems(Update, ratatui_input.map(error))
            .add_systems(Last, ratatui_cleanup.map(error));
    }
}

#[derive(Resource)]
pub struct RatResource {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

fn ratatui_startup(mut commands: Commands) -> io::Result<()> {
    rat_tui::init_panic_hooks();
    let mut terminal = rat_tui::init()?;
    terminal.clear()?;
    commands.insert_resource(RatResource { terminal });

    Ok(())
}

fn ratatui_cleanup(mut events: EventReader<AppExit>) -> io::Result<()> {
    for _ in events.read() {
        rat_tui::restore()?;
    }

    Ok(())
}

#[derive(Event)]
pub struct RatEvent(pub event::Event);

fn ratatui_input(mut event_writer: EventWriter<RatEvent>) -> io::Result<()> {
    if event::poll(std::time::Duration::from_millis(16))? {
        event_writer.send(RatEvent(event::read()?));
    }
    Ok(())
}
