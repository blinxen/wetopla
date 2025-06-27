use crossterm::event::{KeyCode, KeyEvent};
use crossterm::{cursor, QueueableCommand};
use std::io::{Stdout, Write};

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
}

impl Widget for LineInput {
    fn render(&self, stdout: &mut Stdout, available_area: &Rect) -> Result<(), std::io::Error> {
        let rect = self.rect(available_area);
        border(stdout, &rect, "Insert value here", true)?;
        stdout.queue(cursor::MoveTo(rect.x + 1, rect.y + rect.height - 1))?;
        // TODO: Make this more user friendly by adding a blinking cursor
        // https://docs.rs/crossterm/latest/crossterm/cursor/enum.SetCursorStyle.html#variant.BlinkingBar
        stdout.write_all(self.input.as_bytes())?;
        stdout.queue(cursor::MoveTo(
            rect.x + 1 + self.input.chars().count() as u16,
            rect.y + rect.height - 1,
        ))?;
        Ok(())
    }

    fn rect(&self, available_rect: &Rect) -> Rect {
        // Width and height define max input characters
        // 120 means that we can insert 120 characters

        Rect {
            x: available_rect.width / 4,
            y: available_rect.height / 4,
            width: 120,
            height: 2,
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
