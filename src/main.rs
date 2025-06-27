mod application;
mod event_loop;
mod project;
mod task;
mod terminal_utils;
mod widgets;

use application::TodoApp;

use ratatui::{backend::CrosstermBackend, Terminal};

// Enum that contains all possible input modes
#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Insert,
    Deleting,
    Saving,
    Quitting,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = CrosstermBackend::new(std::io::stdout());
    let terminal = Terminal::new(backend)?;

    // Initialize and start the application
    let mut app = TodoApp::new();
    terminal_utils::prepare_terminal()
        .expect("Error occured when trying to prepare the terminal for the application!");

    // Update panic hook to also cleanup the terminal before panicking
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        terminal_utils::restore_terminal()
            .expect("Error occured when trying to restore the previous state of the terminal");
        default_panic(panic_info);
    }));

    // Try to import projects from data file
    app.import_projects();
    app.run(terminal).await?;

    // Cleanup before exiting application
    terminal_utils::restore_terminal()
        .expect("Error occured when trying to restore the previous state of the terminal");

    Ok(())
}
