use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::{ContentStyle, StyledContent, Stylize};

use crate::buffer::Buffer;
use crate::utils::border;
use crate::widgets::PopupWidget;
use crate::widgets::Rect;
use crate::widgets::Widget;

#[derive(PartialEq)]
enum Button {
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

    fn style_button(label: String, selected: bool) -> StyledContent<String> {
        if selected {
            StyledContent::new(ContentStyle::new().black().on_white(), label)
        } else {
            StyledContent::new(ContentStyle::new().reset().on_white(), label)
        }
    }
}

impl Widget for MessageBox {
    fn render(&self, buffer: &mut Buffer, available_area: &Rect) {
        let area = self.rect(available_area);
        // Draw border
        border(buffer, &area, true, String::new(), None, None);
        // Draw message box
        buffer.write_string(
            area.x + (area.width / 2) - (self.question.len() as u16 / 2),
            area.y + area.height / 3,
            self.question.clone().yellow(),
        );
        buffer.write_string(
            area.x + 10,
            area.y + (area.height * 80 / 100),
            Self::style_button(String::from("Yes"), self.selected == Button::Ok),
        );
        buffer.write_string(
            area.x + area.width - 10,
            area.y + (area.height * 80 / 100),
            Self::style_button(String::from("No"), self.selected == Button::No),
        );
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
