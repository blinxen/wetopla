use crate::application::TodoApp;
use crate::buffer::Buffer;
use crate::task::Task;
use crate::utils::{border, Rect};
use crate::widgets::{ContainerWidget, Widget};
use chrono::Local;
use crossterm::style::Stylize;
use serde::{Deserialize, Serialize};
use std::{fs, process::Command};

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
        let task_to_edit = self.tasks.get_mut(task_index).unwrap();

        if !task_to_edit.content.is_empty() {
            // Prefill edit file with the already existing content
            fs::write(&tmp_file, &task_to_edit.content).expect(
                "Unexpected error when trying to write to the file that is used for editing",
            );
        }

        Command::new("vim")
            .arg(&tmp_file)
            .status()
            .expect("Could not find editor! Please install vim!");

        // Actually edit the content of the current task
        if let Ok(content) = fs::read_to_string(&tmp_file) {
            task_to_edit.content = content;
            fs::remove_file(tmp_file)
                .expect("Unexpected error when trying to delete file that is used for editing");
        }

        true
    }

    pub fn toggle_task_done(&mut self, task_index: usize) {
        let task = self.tasks.get_mut(task_index).unwrap();
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
    fn rect(&self, available_area: &Rect) -> Rect {
        available_area.clone()
    }

    fn render(&self, buffer: &mut Buffer, available_area: &Rect) {
        let area = self.rect(available_area);
        border(
            buffer,
            &area,
            self.is_focused(),
            String::from("Projects"),
            None,
            None,
        );
        for (i, project) in self.projects.iter().enumerate() {
            let mut styled_project = format!(
                "{}: {}",
                i,
                if project.title.len() > area.width as usize - 5 {
                    project.title[..area.width as usize - 5].to_owned()
                } else {
                    project.title.to_owned()
                }
            )
            .white();
            if i == self.selected {
                styled_project = styled_project.black().on_white();
            }
            buffer.write_string(area.x + 1, area.y + 1 + i as u16, styled_project);
        }
    }
}
