use std::{
    io::{self, stdout, Stdout},
    panic,
    time::Duration,
};

use bevy::{app::AppExit, prelude::*, utils::error};
use crossterm::{
    cursor,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEventKind, KeyModifiers,
        KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    terminal::{
        disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};

pub struct RatatuiPlugin;

impl Plugin for RatatuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RatatuiEvent>()
            .add_systems(Startup, terminal_setup_system.map(error))
            .add_systems(Update, terminal_event_system.map(error));
    }
}

fn terminal_setup_system(mut commands: Commands) -> io::Result<()> {
    commands.insert_resource(RatatuiContext::init()?);

    Ok(())
}

#[derive(Resource, Deref, DerefMut)]
pub struct RatatuiContext {
    #[deref]
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
    pub kitty_enabled: bool,
    pub mouse_enabled: bool,
}

impl RatatuiContext {
    pub fn init() -> io::Result<Self> {
        Self::init_panic_hook();
        let mut terminal = Self::init_terminal()?;
        let kitty_enabled = RatatuiContext::enable_kitty_protocol().is_ok();
        let mouse_enabled = RatatuiContext::enable_mouse_capture().is_ok();
        terminal.clear()?;
        Ok(Self {
            terminal,
            kitty_enabled,
            mouse_enabled,
        })
    }

    fn init_panic_hook() {
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            let _ = RatatuiContext::restore();
            original_hook(panic_info);
        }));
    }

    fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        Terminal::new(CrosstermBackend::new(stdout()))
    }

    fn enable_kitty_protocol() -> io::Result<()> {
        if !supports_keyboard_enhancement()? {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Kitty keyboard protocol is not supported by this terminal.",
            ));
        }
        stdout().execute(PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::all()))?;
        Ok(())
    }

    fn enable_mouse_capture() -> io::Result<()> {
        stdout().execute(EnableMouseCapture)?;
        Ok(())
    }

    pub fn restore() -> io::Result<()> {
        stdout()
            .execute(PopKeyboardEnhancementFlags)?
            .execute(DisableMouseCapture)?
            .execute(LeaveAlternateScreen)?
            .execute(cursor::Show)?;
        disable_raw_mode()?;
        Ok(())
    }
}

impl Drop for RatatuiContext {
    fn drop(&mut self) {
        let _ = RatatuiContext::restore();
    }
}

#[derive(Debug, Deref, Event, PartialEq, Eq, Clone, Hash)]
pub struct RatatuiEvent(pub event::Event);

pub fn terminal_event_system(
    mut events: EventWriter<RatatuiEvent>,
    mut exit: EventWriter<AppExit>,
) -> io::Result<()> {
    while event::poll(Duration::ZERO)? {
        let ev = event::read()?;
        if let event::Event::Key(key_event) = ev {
            if key_event.kind == KeyEventKind::Press
                && key_event.modifiers == KeyModifiers::CONTROL
                && key_event.code == KeyCode::Char('c')
            {
                exit.send_default();
            }
        }
        events.send(RatatuiEvent(ev));
    }
    Ok(())
}
