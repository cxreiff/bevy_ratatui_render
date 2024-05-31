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
            .add_systems(Update, ratatui_input.map(error));
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct RatContext {
    #[deref]
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
    pub kitty_enabled: bool,
}

impl RatContext {
    pub fn init() -> io::Result<Self> {
        rat_tui::init_panic_hooks();
        let mut terminal = rat_tui::init()?;
        let kitty_enabled = rat_tui::init_kitty_protocol().is_ok();
        terminal.clear()?;

        Ok(RatContext {
            terminal,
            kitty_enabled,
        })
    }
}

impl Drop for RatContext {
    fn drop(&mut self) {
        rat_tui::restore().unwrap();
    }
}

fn ratatui_startup(mut commands: Commands) -> io::Result<()> {
    commands.insert_resource(RatContext::init()?);

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
