mod application;
mod event_loop;
mod project;
mod task;
mod terminal;
mod utils;
mod widgets;

use application::TodoApp;
use std::io::stdout;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut stdout = stdout();
    terminal::prepare_terminal(&mut stdout)?;
    let mut app = TodoApp::new(&mut stdout);
    app.import_projects();
    app.run().await?;
    terminal::restore_terminal(&mut stdout)?;

    Ok(())
}
