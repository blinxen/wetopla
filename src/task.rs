use chrono::{DateTime, Local};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, Widget},
};
use serde::{Deserialize, Serialize};

use crate::{project::Project, widgets::ContainerWidget};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Local>,
    pub modified_at: DateTime<Local>,
    pub done: bool,
}

#[derive(Clone)]
pub struct TaskContainer<'header> {
    tasks: Vec<Task>,
    selected: usize,
    header: Row<'header>,
    focused: bool,
}

impl<'header> TaskContainer<'header> {
    pub fn new(focused: bool) -> Self {
        TaskContainer {
            tasks: Vec::new(),
            selected: 0,
            header: Row::new(vec!["Title", "Done", "Created at"])
                .style(Style::default().add_modifier(Modifier::BOLD)),
            focused,
        }
    }

    pub fn set_project(&mut self, project: Option<&mut Project>) {
        // FIXME: Make this more user friendly
        // At the moment this always sets the focus to 0 if a new task has been added
        self.selected = 0;
        self.tasks = if let Some(project) = project {
            project.tasks.clone()
        } else {
            Vec::new()
        };
    }

    // Return the index of  the currently selected task
    pub fn selected(&self) -> usize {
        self.selected
    }

    // Return the number of the current tasks
    pub fn len(&self) -> usize {
        self.tasks.len()
    }
}

impl<'header> ContainerWidget for TaskContainer<'header> {
    fn move_up(&mut self) {
        if !self.tasks.is_empty() {
            if self.selected != 0 {
                self.selected -= 1;
            } else {
                self.selected = self.tasks.len() - 1;
            }
        }
    }

    fn move_down(&mut self) {
        if !self.tasks.is_empty() {
            if self.selected != (self.tasks.len() - 1) {
                self.selected += 1;
            } else {
                self.selected = 0;
            }
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focus(&mut self, focus: bool) {
        self.focused = focus;
    }
}

impl<'header> Widget for TaskContainer<'header> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut border_style = Style::default();
        if self.is_focused() {
            border_style = border_style.fg(Color::Yellow);
        }

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);

        Table::new(
            self.tasks
                .iter()
                .enumerate()
                .map(|(i, task)| {
                    let mut style = Style::default();
                    if i == self.selected {
                        style = Style::default().fg(Color::Black).bg(Color::White);
                    }

                    Row::new(vec![
                        task.title.clone(),
                        task.done.to_string(),
                        task.created_at.format("%d.%m.%Y %H:%M:%S").to_string(),
                    ])
                    .style(style)
                })
                .collect::<Vec<Row>>(),
        )
        .header(self.header)
        .widths(&[
            Constraint::Length(120),
            Constraint::Length(10),
            Constraint::Length(20),
        ])
        .block(
            Block::default()
                .title("Tasks")
                .borders(Borders::ALL)
                .border_style(border_style)
                .title_alignment(Alignment::Left),
        )
        .render(layout[0], buf);

        let content = if let Some(task) = self.tasks.get(self.selected) {
            task.content.to_owned()
        } else {
            String::new()
        };
        Paragraph::new(content)
            .block(
                Block::default()
                    .title("Content")
                    .borders(Borders::ALL)
                    .title_alignment(Alignment::Left),
            )
            .render(layout[1], buf)
    }
}
