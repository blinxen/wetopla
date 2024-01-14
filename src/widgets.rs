pub mod line_input;
pub mod message_box;

use std::io::Stdout;

use crate::utils::Rect;
use crossterm::event::KeyEvent;

pub trait Widget {
    // Render the widget
    // `available_area` is the whole area that the widget can occupy
    fn render(&self, stdout: &mut Stdout, available_area: &Rect) -> Result<(), std::io::Error>;
    // Calculate the area that the widget will occupy
    fn rect(&self, available_area: &Rect) -> Rect;
}

// A ContainerWidget is a widget that holds a list of items
// For now we assume that all items are listed veritcally
pub trait ContainerWidget {
    // Move selection up
    fn move_up(&mut self);
    // Move selection down
    fn move_down(&mut self);
    // Check if the current widget is focused
    fn is_focused(&self) -> bool;
    // Toogle widget focus, this is only used for styling puposes
    fn toogle_focus(&mut self);
    // Declare that this widget is focused, this is only used for styling puposes
    fn set_focus(&mut self, focus: bool);
}

// A PopupWidget is basically a message box, similar to QMessageBox from QT
pub trait PopupWidget {
    // Process a key event
    fn process_input(&mut self, key_event: &KeyEvent);
    // Close the widget and save the user input
    fn close(&mut self);
}
