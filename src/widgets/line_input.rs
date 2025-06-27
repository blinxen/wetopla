use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::widgets::PopupWidget;

pub struct LineInput {
    input: String,
    visible: bool,
}

impl LineInput {
    pub fn get_input(&self) -> String {
        self.input.to_owned()
    }
}

impl PopupWidget for LineInput {
    fn new() -> Self {
        LineInput {
            input: String::new(),
            visible: false,
        }
    }

    fn render<B: ratatui::backend::Backend>(&self, frame: &mut Frame<B>) {
        let size = self.size(frame.size());
        frame.render_widget(
            Paragraph::new(Text::from(self.input.clone()))
                .style(Style::default().fg(Color::Yellow))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Insert Title")
                        .title_alignment(Alignment::Center),
                ),
            size,
        );
        frame.set_cursor(size.x + self.input.chars().count() as u16 + 1, size.y + 1);
    }

    fn size(&self, available_rect: Rect) -> Rect {
        // Width and height define max input characters
        // 120 means that we can insert 120 characters

        Rect::new(available_rect.width / 4, available_rect.height / 4, 120, 3)
    }

    fn process_input(&mut self, key_event: &KeyEvent) {
        // visible was introduced to be able to differentiate between the
        // line_input being displayed and user input after displaying the line_input
        if self.visible {
            match key_event.code {
                KeyCode::Char(char) => self.input.push(char),
                KeyCode::Backspace => {
                    self.input.pop();
                }
                // KeyCode::Left => todo!(),
                // KeyCode::Right => todo!(),
                // KeyCode::Up => todo!(),
                // KeyCode::Down => todo!(),
                // KeyCode::Home => todo!(),
                // KeyCode::End => todo!(),
                // KeyCode::Delete => todo!(),
                _ => {}
            }
        } else {
            self.visible = true;
        }
    }

    fn close(&mut self) {
        self.visible = false;
        self.input.clear();
    }
}
