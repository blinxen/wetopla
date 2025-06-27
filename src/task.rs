use chrono::{DateTime, Local};
use crossterm::style::Color;
use crossterm::style::Stylize;
use crossterm::{style, QueueableCommand};
use serde::{Deserialize, Serialize};
use std::io::Stdout;
use std::io::Write;

use crate::project::Project;
use crate::utils::border;
use crate::utils::build_row;
use crate::utils::go_to_next_line_in_area;
use crate::utils::reset_cursor_in_area;
use crate::utils::split_rect_by_height;
use crate::utils::Rect;
use crate::widgets::ContainerWidget;
use crate::widgets::Widget;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Local>,
    pub modified_at: DateTime<Local>,
    pub done: bool,
}

#[derive(Clone)]
pub struct TaskContainer {
    tasks: Vec<Task>,
    selected: usize,
    focused: bool,
}

impl TaskContainer {
    pub fn new(focused: bool) -> Self {
        TaskContainer {
            tasks: Vec::new(),
            selected: 0,
            focused,
        }
    }

    pub fn set_project(&mut self, project: Option<&mut Project>) {
        if self.tasks.len() < self.selected {
            self.selected = self.tasks.len().saturating_sub(1);
        }
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

impl ContainerWidget for TaskContainer {
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

    fn toogle_focus(&mut self) {
        self.focused = !self.focused;
    }
}

impl Widget for TaskContainer {
    fn rect(&self, available_area: &Rect) -> Rect {
        available_area.clone()
    }

    fn render(&self, stdout: &mut Stdout, available_area: &Rect) -> Result<(), std::io::Error> {
        let mut selected_task_content = String::new();
        // Calculate list and content areas
        let (task_list_area, task_content_area) = split_rect_by_height(&self.rect(available_area));
        // Draw border
        border(stdout, &task_list_area, "Tasks", self.is_focused())?;
        reset_cursor_in_area(stdout, &task_list_area)?;
        let done_column_space = 10;
        let created_at_column_space = 19;
        let title_column_space = (task_list_area.width as f32 * 0.95) as u16
            - created_at_column_space
            - done_column_space;
        let styled_project = build_row(vec![
            ("Title", title_column_space as usize),
            ("Done", done_column_space as usize),
            ("Created At", created_at_column_space as usize),
        ]);
        go_to_next_line_in_area(stdout, &task_list_area, 1)?;
        stdout.queue(style::PrintStyledContent(styled_project.bold()))?;
        for (i, task) in self.tasks.iter().enumerate() {
            go_to_next_line_in_area(stdout, &task_list_area, 1)?;
            let mut styled_project = build_row(vec![
                (&task.title, title_column_space as usize),
                (&task.done.to_string(), done_column_space as usize),
                (
                    &task.created_at.format("%d.%m.%Y %H:%M:%S").to_string(),
                    created_at_column_space as usize,
                ),
            ]);
            if i == self.selected {
                selected_task_content = task.content.clone();
                styled_project = styled_project.on(Color::White).with(Color::Black);
            }
            stdout.queue(style::PrintStyledContent(styled_project))?;
        }

        border(stdout, &task_content_area, "Content", false)?;
        reset_cursor_in_area(stdout, &task_content_area)?;
        // Buffer used to store the string that will be written to stdout
        let mut content_to_write: String;
        // Max line length for this container
        // We subtract 2 here because one column is used for the outer border and one for the
        // container border
        let max_content_line_length = task_content_area.width as usize - 2;
        for line in selected_task_content.lines() {
            // We need to convert &str to a string since we want to do some operations on it
            let mut owned_line = line.to_owned();
            loop {
                // If the line fits on the first line then write the full line
                if !owned_line.is_empty() && owned_line.len() < max_content_line_length {
                    // We clone the string here because we store everything we want to write in
                    // content_to_write
                    content_to_write = owned_line.clone();
                    owned_line.clear();
                // If the line is not empty and still does not fit on one line, then split it and
                // continue with the loop
                } else if owned_line.len() > max_content_line_length {
                    // TODO: Don't split words, we should handle slices that split words
                    // The correct way to handle this is to search for a whitespace at
                    // max_content_line_length or before and split there
                    content_to_write = owned_line[..max_content_line_length].to_string();
                    owned_line = owned_line[max_content_line_length..].to_string();
                // If the line is not empty but the remaining characters fit on one line, then
                // write it to stdout and break from the loop
                // Break from the loop since we don't have anything to print anymore
                } else {
                    break;
                }
                go_to_next_line_in_area(stdout, &task_content_area, 1)?;
                stdout.write_all(content_to_write.as_bytes())?;
            }
        }

        Ok(())
    }
}
