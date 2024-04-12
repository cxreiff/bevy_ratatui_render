use bevy::{app::AppExit, prelude::*};
use crossterm::event;
use ratatui::prelude::*;
use std::io::{Result, Stdout};

mod tui;

#[derive(Resource)]
pub struct RatatuiResource {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

pub fn ratatui_plugin(app: &mut App) {
    app.add_event::<RatatuiEvent>()
        .add_systems(Startup, ratatui_startup.pipe(ratatui_error_handler))
        .add_systems(Update, ratatui_input.pipe(ratatui_error_handler))
        .add_systems(Last, ratatui_cleanup.pipe(ratatui_error_handler));
}

fn ratatui_startup(mut commands: Commands) -> Result<()> {
    let mut terminal = tui::init()?;
    terminal.clear()?;
    commands.insert_resource(RatatuiResource { terminal });

    Ok(())
}

fn ratatui_cleanup(mut events: EventReader<AppExit>) -> Result<()> {
    for _ in events.read() {
        tui::restore()?;
    }

    Ok(())
}

#[derive(Event)]
pub struct RatatuiEvent(pub event::Event);

fn ratatui_input(mut event_writer: EventWriter<RatatuiEvent>) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(16))? {
        event_writer.send(RatatuiEvent(event::read()?));
    }
    Ok(())
}

pub fn ratatui_error_handler(In(result): In<Result<()>>) {
    if let Err(err) = result {
        println!("encountered an error {:?}", err);
    }
}
