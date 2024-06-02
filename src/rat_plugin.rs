use std::{
    io::{self, Stdout},
    time::Duration,
};

use bevy::{app::AppExit, prelude::*, utils::error};
use crossterm::event::{self, KeyCode, KeyModifiers};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::rat_tui;

pub struct RatatuiPlugin;

impl Plugin for RatatuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RatatuiEvent>()
            .add_systems(Startup, ratatui_startup.map(error))
            .add_systems(Update, ratatui_input.map(error));
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct RatatuiContext {
    #[deref]
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
    pub kitty_enabled: bool,
}

impl RatatuiContext {
    pub fn init() -> io::Result<Self> {
        rat_tui::init_panic_hooks();
        let mut terminal = rat_tui::init()?;
        let kitty_enabled = rat_tui::init_kitty_protocol().is_ok();
        terminal.clear()?;

        Ok(Self {
            terminal,
            kitty_enabled,
        })
    }
}

impl Drop for RatatuiContext {
    fn drop(&mut self) {
        rat_tui::restore().unwrap();
    }
}

fn ratatui_startup(mut commands: Commands) -> io::Result<()> {
    commands.insert_resource(RatatuiContext::init()?);

    Ok(())
}

#[derive(Event)]
pub struct RatatuiEvent(pub event::Event);

fn ratatui_input(
    mut exit: EventWriter<AppExit>,
    mut rat_event: EventWriter<RatatuiEvent>,
) -> io::Result<()> {
    while event::poll(Duration::ZERO)? {
        let ratatui_event = event::read()?;

        if let event::Event::Key(key_event) = ratatui_event {
            if key_event.modifiers == KeyModifiers::CONTROL && key_event.code == KeyCode::Char('c')
            {
                exit.send_default();
            }
        }

        rat_event.send(RatatuiEvent(ratatui_event));
    }
    Ok(())
}
