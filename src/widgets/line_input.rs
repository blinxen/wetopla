use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::Stylize;

use crate::buffer::Buffer;
use crate::utils::border;
use crate::widgets::PopupWidget;
use crate::widgets::Rect;
use crate::widgets::Widget;

pub struct LineInput {
    input: String,
    visible: bool,
}

impl LineInput {
    pub fn new() -> Self {
        LineInput {
            input: String::new(),
            visible: false,
        }
    }

    pub fn value(&self) -> String {
        self.input.to_owned()
    }

    pub fn set_value(&mut self, input: String) {
        self.input = input;
    }
}

impl Widget for LineInput {
    fn render(&self, buffer: &mut Buffer, available_area: &Rect) {
        let area = self.rect(available_area);
        border(
            buffer,
            &area,
            true,
            String::from("Insert value here"),
            None,
            None,
        );
        // TODO: Make this more user friendly by adding a blinking cursor
        // https://docs.rs/crossterm/latest/crossterm/cursor/enum.SetCursorStyle.html#variant.BlinkingBar
        buffer.write_string(
            area.x + 1,
            area.y + area.height - 2,
            self.input[self.input.len().saturating_sub(area.width as usize - 2)..]
                .to_string()
                .reset(),
        );
    }

    fn rect(&self, available_rect: &Rect) -> Rect {
        // Width and height define max input characters
        // 120 means that we can insert 120 characters

        Rect {
            x: available_rect.width / 4,
            y: available_rect.height / 4,
            width: 120,
            height: 3,
        }
    }
}

impl PopupWidget for LineInput {
    fn process_input(&mut self, key_event: &KeyEvent) {
        // visible was introduced to be able to differentiate between the
        // line_input being displayed and user input after displaying the line_input
        if self.visible {
            match key_event.code {
                KeyCode::Char(char) => self.input.push(char),
                KeyCode::Backspace => drop(self.input.pop()),
                KeyCode::Esc => {
                    self.visible = false;
                    self.input.clear();
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
