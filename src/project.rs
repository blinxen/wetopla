use std::{fs, process::Command};

use chrono::Local;
use serde::{Deserialize, Serialize};
use tui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Widget},
};

use crate::{application::TodoApp, task::Task, widgets::ContainerWidget};

// A project contains a list of tasks
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub title: String,
    pub tasks: Vec<Task>,
    pub done: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectContainer {
    projects: Vec<Project>,
    selected: usize,
    focused: bool,
}

impl Project {
    pub fn add_task(&mut self, task_title: String) {
        self.tasks.push(Task {
            title: task_title,
            content: String::new(),
            created_at: Local::now(),
            modified_at: Local::now(),
            done: false,
        });
    }

    pub fn remove_selected_task(&mut self, task_index: usize) {
        if !self.tasks.is_empty() {
            self.tasks.remove(task_index);
        }
    }

    pub fn edit_task(&mut self, task_index: usize) -> bool {
        let tmp_file = TodoApp::data_directory_path().join("task.edit");

        if self.tasks.is_empty() {
            // Don't do anything if there are not tasks
            return false;
        }
        // Get task that we want to edit
        let mut task_to_edit = self.tasks.get_mut(task_index).unwrap();

        if !task_to_edit.content.is_empty() {
            // Prefill edit file with the already existing content
            fs::write(&tmp_file, &task_to_edit.content).expect(
                "Unexpected error when trying to write to the file that is used for editing",
            );
        }

        Command::new("nvim")
            .arg(&tmp_file)
            .status()
            .expect("Could not find editor! Please install neovim!");

        // Actually edit the content of the current task
        if let Ok(content) = fs::read_to_string(&tmp_file) {
            task_to_edit.content = content;
            fs::remove_file(tmp_file)
                .expect("Unexpected error when trying to delete file that is used for editing");
        }

        true
    }

    pub fn toggle_task_done(&mut self, task_index: usize) {
        let mut task = self.tasks.get_mut(task_index).unwrap();
        task.done = !task.done;
    }
}

impl ProjectContainer {
    pub fn new(focused: bool) -> Self {
        ProjectContainer {
            projects: Vec::new(),
            selected: 0,
            focused,
        }
    }

    pub fn add_project(&mut self, project_name: String) {
        if !project_name.is_empty() {
            self.projects.push(Project {
                title: project_name,
                tasks: Vec::new(),
                done: false,
            });
            self.selected = self.projects.len() - 1;
        }
    }

    pub fn remove_selected_project(&mut self) {
        if !self.projects.is_empty() {
            let selected_ = self.selected;

            if self.selected <= 1 {
                self.selected = 0
            } else {
                self.selected -= 1;
            }
            self.projects.remove(selected_);
        }
    }

    pub fn current_project(&mut self) -> Option<&mut Project> {
        self.projects.get_mut(self.selected)
    }
}

impl ContainerWidget for ProjectContainer {
    fn move_up(&mut self) {
        if !self.projects.is_empty() {
            if self.selected != 0 {
                self.selected -= 1;
            } else {
                self.selected = self.projects.len() - 1;
            }
        }
    }

    fn move_down(&mut self) {
        if !self.projects.is_empty() {
            if self.selected != (self.projects.len() - 1) {
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

impl Widget for ProjectContainer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut border_style = Style::default();
        if self.is_focused() {
            border_style = border_style.fg(Color::Yellow);
        }

        let projects: Vec<ListItem> = self
            .projects
            .iter()
            .enumerate()
            .map(|(i, project)| {
                let mut style = Style::default();
                if i == self.selected {
                    style = Style::default().fg(Color::Black).bg(Color::White);
                }

                let content = vec![Spans::from(Span::styled(
                    format!("{}: {}", i, project.title),
                    style,
                ))];
                ListItem::new(content)
            })
            .collect();

        List::new(projects)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title(Span::styled(
                        "Projects",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .title_alignment(Alignment::Left),
            )
            .render(area, buf);
    }
}
