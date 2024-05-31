use std::io::{self, stdout, Stdout};
use std::panic;

use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::{
    cursor, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crossterm::{queue, terminal};
use ratatui::prelude::*;

pub fn init() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn init_panic_hooks() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = restore();
        original_hook(panic_info);
    }));
}

pub fn init_kitty_protocol() -> io::Result<()> {
    if let Ok(supported) = terminal::supports_keyboard_enhancement() {
        if supported {
            execute!(
                stdout(),
                PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::all())
            )?;

            return Ok(());
        }
    }

    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "kitty keyboard protocol is not supported by this terminal.",
    ))
}

pub fn restore() -> io::Result<()> {
    if terminal::supports_keyboard_enhancement().is_ok() {
        queue!(stdout(), PopKeyboardEnhancementFlags)?;
    }
    execute!(stdout(), PopKeyboardEnhancementFlags)?;
    execute!(stdout(), LeaveAlternateScreen)?;
    execute!(stdout(), cursor::Show)?;
    disable_raw_mode()?;
    Ok(())
}
