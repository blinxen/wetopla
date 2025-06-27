use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::{self, Color};
use crossterm::{cursor, QueueableCommand};
use std::io::{Stdout, Write};

use crate::utils::border;
use crate::utils::highlight_button_text;
use crate::widgets::PopupWidget;
use crate::widgets::Rect;
use crate::widgets::Widget;

#[derive(PartialEq)]
pub enum Button {
    Ok,
    No,
}

pub struct MessageBox {
    question: String,
    visible: bool,
    selected: Button,
}

impl MessageBox {
    pub fn new() -> Self {
        MessageBox {
            question: String::new(),
            visible: false,
            selected: Button::No,
        }
    }

    pub fn accepted(&self) -> bool {
        self.selected == Button::Ok
    }

    pub fn set_question(&mut self, text: &str) {
        self.question = text.to_owned();
    }
}

impl Widget for MessageBox {
    fn render(&self, stdout: &mut Stdout, available_area: &Rect) -> Result<(), std::io::Error> {
        let rect = self.rect(available_area);
        // Draw border
        border(stdout, &rect, "", true)?;
        stdout.queue(cursor::MoveTo(
            rect.x + (rect.width / 2) - (self.question.len() as u16 / 2),
            rect.y + rect.height / 3,
        ))?;
        // Draw message box
        stdout.queue(style::SetForegroundColor(Color::Yellow))?;
        stdout.write_all(self.question.as_bytes())?;
        stdout.queue(style::SetForegroundColor(Color::Reset))?;
        stdout.queue(cursor::MoveTo(
            rect.x + 10,
            rect.y + (rect.height * 80 / 100),
        ))?;
        highlight_button_text(stdout, "Yes", self.selected == Button::Ok)?;
        stdout.queue(cursor::MoveTo(
            rect.x + rect.width - 10,
            rect.y + (rect.height * 80 / 100),
        ))?;
        highlight_button_text(stdout, "No", self.selected == Button::No)?;

        Ok(())
    }

    fn rect(&self, available_rect: &Rect) -> Rect {
        // Width and height define max input characters
        // 120 means that we can insert 120 characters

        Rect {
            x: available_rect.width / 4,
            y: available_rect.height / 4,
            width: 100,
            height: 7,
        }
    }
}

impl PopupWidget for MessageBox {
    fn process_input(&mut self, key_event: &KeyEvent) {
        // visible was introduced to be able to differentiate between the
        // line_input being displayed and user input after displaying the line_input
        if self.visible {
            if key_event.code == KeyCode::Tab {
                self.selected = if self.selected == Button::Ok {
                    Button::No
                } else {
                    Button::Ok
                };
            }
        } else {
            self.visible = true;
        }
    }

    fn close(&mut self) {
        self.visible = false;
        self.selected = Button::No;
        self.question = String::new();
    }
}
