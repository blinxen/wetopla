mod application;
mod buffer;
mod event_loop;
mod project;
mod task;
mod terminal;
mod utils;
mod widgets;

use application::TodoApp;

fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        terminal::restore_terminal().expect("Could no restore terminal");
        hook(info);
    }));
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    set_panic_hook();
    terminal::prepare_terminal()?;
    let mut app = TodoApp::new();
    app.import_projects();
    app.run().await?;
    terminal::restore_terminal()?;

    Ok(())
}
