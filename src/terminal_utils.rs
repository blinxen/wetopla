use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

// Enable some terminal features that will be required for this app to work
pub fn prepare_terminal() -> Result<(), Box<dyn std::error::Error>> {
    // Disable some default Terminal features that are not required
    enable_raw_mode()?;
    // Enable mouse support and make sure terminal starts in an alternate screen
    // Alternate means that the current screen is restored after exiting the application
    execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    Ok(())
}

// Cleanup work that needs to be done after exiting the application
// This can include reseting the screen, re-enabling some terminal features etc.
pub fn restore_terminal() -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;

    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    execute!(std::io::stdout(), cursor::Show)?;

    Ok(())
}
