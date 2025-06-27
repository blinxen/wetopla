use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::widgets::PopupWidget;

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
    pub fn accepted(&self) -> bool {
        self.selected == Button::Ok
    }

    pub fn set_question(&mut self, text: String) {
        self.question = text
    }
}

impl PopupWidget for MessageBox {
    fn new() -> Self {
        MessageBox {
            question: String::new(),
            visible: false,
            selected: Button::No,
        }
    }

    fn render<B: tui::backend::Backend>(&self, frame: &mut Frame<B>) {
        let size = self.size(frame.size());
        // Render main block
        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
            size,
        );

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .vertical_margin(2)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref())
            .split(size);

        frame.render_widget(
            Paragraph::new(self.question.to_owned()).alignment(Alignment::Center),
            layout[0],
        );

        let button_layer = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(layout[1]);

        let mut yes_style = Style::default();
        let mut no_style = Style::default();
        if self.selected == Button::Ok {
            yes_style = Style::default().fg(Color::Black).bg(Color::White);
        } else {
            no_style = Style::default().fg(Color::Black).bg(Color::White);
        }

        frame.render_widget(
            Paragraph::new(Span::styled("Yes", yes_style)).alignment(Alignment::Center),
            button_layer[0],
        );
        frame.render_widget(
            Paragraph::new(Span::styled("No", no_style)).alignment(Alignment::Center),
            button_layer[1],
        );
    }

    fn size(&self, available_rect: Rect) -> Rect {
        // Width and height define max input characters
        // 120 means that we can insert 120 characters

        Rect::new(available_rect.width / 4, available_rect.height / 4, 100, 10)
    }

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
