use crate::utils::Rect;
use crossterm::{QueueableCommand, terminal};
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::io::{Error, stdout};

// Enable some terminal features that will be required for this app to work
pub fn prepare_terminal() -> Result<(), Error> {
    // Disable some default Terminal features that are not required
    enable_raw_mode()?;
    // Enable mouse support and make sure terminal starts in an alternate screen
    // Alternate means that the current screen is restored after exiting the application
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    stdout().queue(cursor::Hide)?;

    Ok(())
}

// Cleanup work that needs to be done after exiting the application
// This can include reseting the screen, re-enabling some terminal features etc.
pub fn restore_terminal() -> Result<(), Error> {
    disable_raw_mode()?;

    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    execute!(stdout(), cursor::Show)?;

    Ok(())
}

// Return the terminal size as a Rect
pub fn size() -> Rect {
    let (width, height) = terminal::size().expect("Could not determine terminal size");

    Rect {
        x: 0,
        y: 0,
        width,
        height,
    }
}
