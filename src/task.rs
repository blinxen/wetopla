use chrono::{DateTime, Local};
use crossterm::style::Stylize;
use serde::{Deserialize, Serialize};

use crate::buffer::Buffer;
use crate::project::Project;
use crate::utils::Rect;
use crate::utils::border;
use crate::utils::build_row;
use crate::utils::split_rect_by_height;
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
}

impl Widget for TaskContainer {
    fn rect(&self, available_area: &Rect) -> Rect {
        available_area.clone()
    }

    fn render(&self, buffer: &mut Buffer, available_area: &Rect) {
        let mut selected_task_content = String::new();

        // Calculate list and content areas
        let (task_list_area, task_content_area) = split_rect_by_height(&self.rect(available_area));
        // Draw border for tasks
        border(
            buffer,
            &task_list_area,
            self.is_focused(),
            String::from("Tasks"),
            None,
            None,
        );

        let done_column_space: usize = 10;
        let created_at_column_space: usize = 19;
        let title_column_space =
            (task_list_area.width - 2) as usize - created_at_column_space - done_column_space;

        // Draw header
        let header = build_row(vec![
            ("Title", title_column_space),
            ("Done", done_column_space),
            ("Created At", created_at_column_space),
        ]);
        buffer.write_string(task_list_area.x + 1, task_list_area.y + 1, header.bold());

        // Draw tasks
        for (i, task) in self.tasks.iter().enumerate() {
            let mut styled_task = build_row(vec![
                (
                    if task.title.len() > title_column_space - 1 {
                        &task.title[..title_column_space - 1]
                    } else {
                        &task.title
                    },
                    title_column_space,
                ),
                (&task.done.to_string(), done_column_space),
                (
                    &task.created_at.format("%d.%m.%Y %H:%M:%S").to_string(),
                    created_at_column_space,
                ),
            ])
            .white();

            if i == self.selected {
                selected_task_content = task.content.clone();
                styled_task = styled_task.black().on_white();
            }

            buffer.write_string(
                task_list_area.x + 1,
                task_list_area.y + 2 + i as u16,
                styled_task,
            );
        }

        // Draw border for task content
        border(
            buffer,
            &task_content_area,
            false,
            String::from("Content"),
            None,
            None,
        );
        // TODO: Don't truncate and make this not just a preview but a scrollable area
        for (i, line) in selected_task_content
            .lines()
            .take(task_content_area.height as usize - 2)
            .enumerate()
        {
            buffer.write_string(
                task_content_area.x + 1,
                task_content_area.y + 1 + i as u16,
                line[..line.len().min(task_content_area.width as usize - 2)]
                    .to_string()
                    .reset(),
            );
        }
    }
}
