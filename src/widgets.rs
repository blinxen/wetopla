use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

pub mod line_input;
pub mod message_box;

// Trait that contains methods that all container widgets should implement
pub trait ContainerWidget {
    // Move selection up
    fn move_up(&mut self);
    // Move selection down
    fn move_down(&mut self);
    // Check if the current widget is focused
    fn is_focused(&self) -> bool;
    // toggle focus of widget
    fn set_focus(&mut self, focus: bool);
}

// Trait for my own custom widgets
// Each widget must define its own method for returning the user input value
pub trait PopupWidget {
    fn new() -> Self;
    // Render the widget
    fn render<B: ratatui::backend::Backend>(&self, frame: &mut Frame<B>);
    // Get the size of a widget
    fn size(&self, available_rect: Rect) -> Rect;
    // Process the current key_event
    fn process_input(&mut self, key_event: &KeyEvent);
    // Close the widget and save the user input
    fn close(&mut self);
}
