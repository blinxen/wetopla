use std::env;
use std::fs::OpenOptions;
use std::io::Stdout;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::event_loop::{Event, EventLoop};
use crate::project::ProjectContainer;
use crate::task::TaskContainer;
use crate::terminal;
use crate::utils::border;
use crate::utils::go_to_next_line_in_area;
use crate::utils::Rect;
use crate::widgets::line_input::LineInput;
use crate::widgets::message_box::MessageBox;
use crate::widgets::Widget;
use crate::widgets::{ContainerWidget, PopupWidget};
use crossterm::cursor::MoveTo;
use crossterm::event::Event as CrosstermEvent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::style;
use crossterm::style::Color;
use crossterm::style::PrintStyledContent;
use crossterm::style::Stylize;
use crossterm::QueueableCommand;

const MAX_LOG_DURATION: u8 = 3;
#[derive(Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Insert,
    Deleting,
    Saving,
    Quitting,
}

// App holds the state of the application
pub struct TodoApp<'a> {
    stdout: &'a mut Stdout,
    // Current input mode
    input_mode: InputMode,
    // Container that will manage and display all projects
    projects: ProjectContainer,
    // Container that will manage and display all tasks of a project
    tasks: TaskContainer,
    // Bool that stores whether the user wants to quit
    // the application or not
    quit: bool,
    // Log message that is written at the bottom
    log_message: String,
    // Counter that keeps track of the duration of the current log message
    log_message_duration: u8,
    // Whether the current state was modified
    dirty: bool,
    // Message box widget
    message_box: MessageBox,
    // Line input wideget
    line_input: LineInput,
    // Event loop that controls draw and crossterm key events
    // TODO: Remove event loop and put it in main
    event_loop: EventLoop,
}

impl<'a> TodoApp<'a> {
    pub fn new(stdout: &'a mut Stdout) -> Self {
        Self {
            stdout,
            input_mode: InputMode::Normal,
            projects: ProjectContainer::new(true),
            tasks: TaskContainer::new(false),
            quit: false,
            log_message: String::new(),
            log_message_duration: 0,
            dirty: false,
            message_box: MessageBox::new(),
            line_input: LineInput::new(),
            event_loop: EventLoop::start(),
        }
    }

    // Main loop
    pub async fn run(&mut self) -> Result<(), std::io::Error> {
        loop {
            match self.event_loop.event_rx.recv().await {
                Some(Event::Draw) => {
                    // Reset log message after 5 Draw events
                    // which equals to 5 seconds
                    if self.log_message_duration == MAX_LOG_DURATION {
                        self.log_message = String::new();
                    }
                    // Increment log duration
                    if !self.log_message.is_empty() {
                        self.log_message_duration += 1;
                    }
                }
                Some(Event::Key(CrosstermEvent::Key(key))) => {
                    self.handle_key_event(&key).await;

                    if self.input_mode == InputMode::Saving
                        || self.input_mode == InputMode::Quitting
                        || self.input_mode == InputMode::Deleting
                    {
                        self.message_box.process_input(&key);
                    } else if self.input_mode == InputMode::Insert {
                        self.line_input.process_input(&key);
                    }
                }
                _ => {}
            };

            // Render app in terminal
            self.render()?;

            if self.quit {
                break;
            }

            self.stdout.flush()?;
        }

        Ok(())
    }

    fn render(&mut self) -> Result<(), std::io::Error> {
        // Render main block
        terminal::clear(self.stdout)?;
        // An area where the application will be drawn
        // This is normally the whole teminal size
        let area = terminal::size()?;
        border(self.stdout, &area, "", false)?;
        // Display projects
        let projects_area = Rect {
            x: area.x + 2,
            y: area.y + 1,
            width: (area.width as f32 * 0.20) as u16,
            // 5 was chosen from: y offset (1) + the number of of lines we want to reserve for (4)
            // other stuff
            height: area.height - 5,
        };
        self.projects.render(self.stdout, &projects_area)?;
        // Display tasks
        let tasks_area = Rect {
            x: projects_area.width + 3,
            y: projects_area.y,
            // 5 was chosen from: x offset (3) + margin left that we want (2)
            width: area.width - projects_area.width - 5,
            height: projects_area.height,
        };
        self.tasks.render(self.stdout, &tasks_area)?;
        // Draw mode
        let mode = match &self.input_mode {
            InputMode::Normal => ("INPUT", Color::Cyan),
            InputMode::Insert => ("INSERT", Color::Green),
            InputMode::Saving => ("SAVING", Color::Magenta),
            InputMode::Quitting => ("QUITTING", Color::Grey),
            InputMode::Deleting => ("DELETING", Color::Grey),
        };
        self.stdout.queue(MoveTo(area.x + 2, area.height - 3))?;
        self.stdout.queue(style::SetForegroundColor(Color::Black))?;
        self.stdout.queue(style::SetBackgroundColor(mode.1))?;
        self.stdout.write_all(mode.0.as_bytes())?;
        self.stdout.write_all(
            " ".repeat((projects_area.width + tasks_area.width) as usize - mode.0.len())
                .as_bytes(),
        )?;
        self.stdout.queue(style::SetBackgroundColor(Color::Reset))?;

        // Render log bar
        go_to_next_line_in_area(self.stdout, &area, 2)?;
        self.stdout.queue(PrintStyledContent(
            self.log_message
                .clone()
                .stylize()
                .on(Color::Reset)
                .with(Color::White),
        ))?;

        if self.input_mode == InputMode::Saving
            || self.input_mode == InputMode::Quitting
            || self.input_mode == InputMode::Deleting
        {
            self.message_box.render(self.stdout, &area)?;
        }

        if self.input_mode == InputMode::Insert {
            self.line_input.render(self.stdout, &area)?;
        }

        Ok(())
    }

    async fn handle_key_event(&mut self, key: &KeyEvent) {
        // Handle keys with modifiers
        if self.input_mode == InputMode::Normal {
            if let KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } = key
            {
                if self.dirty {
                    self.input_mode = InputMode::Saving;
                    self.message_box
                        .set_question("Are you sure that you want to save?");
                }
            }
        }

        // Handle keys without modifier
        // TODO: Why do we need to borrow here?
        match &self.input_mode {
            InputMode::Normal => {
                match key.code {
                    KeyCode::Char('q') => {
                        if self.dirty {
                            self.input_mode = InputMode::Quitting;
                            self.message_box
                                .set_question("Do you want to save your changes before quitting?");
                        } else {
                            self.quit = true
                        }
                    }
                    KeyCode::Char('i') => self.input_mode = InputMode::Insert,
                    // Navigate between items
                    KeyCode::Up => {
                        if self.projects.is_focused() {
                            self.projects.move_up();
                            self.update_tasks();
                        } else {
                            self.tasks.move_up();
                        }
                    }
                    KeyCode::Down => {
                        if self.projects.is_focused() {
                            self.projects.move_down();
                            self.update_tasks();
                        } else {
                            self.tasks.move_down();
                        }
                    }
                    KeyCode::Enter => {
                        if self.projects.is_focused() && self.projects.current_project().is_some() {
                            self.projects.set_focus(false);
                            self.tasks.set_focus(true);
                        }
                    }
                    KeyCode::Esc => {
                        if self.tasks.is_focused() {
                            self.projects.set_focus(true);
                            self.tasks.set_focus(false);
                        }
                    }
                    KeyCode::Char('e') => {
                        if self.tasks.is_focused() {
                            self.dirty = true;
                            self.edit_task(self.tasks.selected()).await;
                        }
                    }
                    KeyCode::Delete => {
                        self.input_mode = InputMode::Deleting;
                        self.message_box
                            .set_question("Are you sure that you want to delete?");
                    }
                    KeyCode::Char('d') => {
                        if self.tasks.is_focused() {
                            self.dirty = true;
                            self.projects
                                .current_project()
                                .unwrap()
                                .toggle_task_done(self.tasks.selected());
                            self.update_tasks();
                        }
                    }
                    _ => {}
                }
            }
            InputMode::Insert => match key.code {
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.line_input.close();
                }
                KeyCode::Enter => {
                    // Min project / task title is 3
                    if !self.line_input.value().is_empty() || self.line_input.value().len() > 3 {
                        self.input_mode = InputMode::Normal;
                        self.dirty = true;

                        if self.projects.is_focused() {
                            // Add project to projects list
                            self.projects.add_project(self.line_input.value());
                        } else {
                            // Add task to project and open editor to write the content of the task
                            self.projects
                                .current_project()
                                .unwrap()
                                .add_task(self.line_input.value());
                            self.edit_task(self.tasks.len()).await;
                        }
                        self.update_tasks();
                        self.line_input.close();
                    }
                }
                _ => {}
            },
            InputMode::Deleting => match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.input_mode = InputMode::Normal;

                    if key.code == KeyCode::Enter && self.message_box.accepted() {
                        self.dirty = true;
                        if self.projects.is_focused() {
                            self.projects.remove_selected_project();
                        } else {
                            self.projects
                                .current_project()
                                .unwrap()
                                .remove_selected_task(self.tasks.selected());
                            self.update_tasks();
                        }
                    }

                    self.message_box.close();
                }
                _ => {}
            },
            InputMode::Saving => match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.input_mode = InputMode::Normal;

                    if key.code == KeyCode::Enter && self.message_box.accepted() {
                        self.save();
                    }

                    self.message_box.close();
                }
                _ => {}
            },
            InputMode::Quitting => match key.code {
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.message_box.close();
                }
                KeyCode::Enter => {
                    if self.message_box.accepted() {
                        self.save();
                    }
                    self.quit = true;
                }
                _ => {}
            },
        }
    }

    fn log(&mut self, message: String) {
        self.log_message = message;
        self.log_message_duration = 0;
    }

    async fn edit_task(&mut self, task_index: usize) {
        // Cancel event loop
        // We cancel it because we will temporarily leave the application and enter
        // the external text editor
        self.event_loop.abort.cancel();
        // Wait until all tasks are finished
        loop {
            if self.event_loop.event_rx.recv().await.is_none() {
                break;
            }
        }
        // TODO: Remove expect
        terminal::restore_terminal(self.stdout)
            .expect("Error occured when trying to restore the previous state of the terminal!");
        // TODO: Think about adding some error handling here
        // Maybe display error in the log bar
        self.projects
            .current_project()
            .unwrap()
            .edit_task(task_index);
        terminal::prepare_terminal(self.stdout)
            .expect("Error occured when trying to prepare the terminal for the application!");
        // Restart event loop after entering the application
        self.event_loop = EventLoop::start();
        self.update_tasks();
    }

    // Update tasks in task container
    fn update_tasks(&mut self) {
        self.tasks.set_project(self.projects.current_project());
    }

    pub fn data_directory_path() -> PathBuf {
        let home_path = env::var_os("HOME").unwrap();
        Path::new(&home_path).join(".weeklyplaner")
    }

    fn data_path(&self) -> PathBuf {
        Self::data_directory_path().join("data.json")
    }

    fn save(&mut self) {
        // Create data directory
        // If the weeklyplane directory could not be created then just panic
        std::fs::create_dir_all(Self::data_directory_path()).unwrap();
        // Open / create data file
        let data_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.data_path());
        // Only try to write if file could be created or exists
        if let Ok(data_file) = data_file {
            serde_json::to_writer(data_file, &self.projects)
                .or_else(|x| {
                    // Print error to log bar
                    self.log(x.to_string());
                    Ok::<(), ()>(())
                })
                .unwrap();
            self.log("Saved".to_string());
        } else {
            self.log(data_file.err().unwrap().to_string());
        }

        self.dirty = false;
    }

    pub fn import_projects(&mut self) {
        let data_file = OpenOptions::new().read(true).open(self.data_path());

        if let Ok(data_file) = data_file {
            if let Ok(reader) = serde_json::from_reader(data_file) {
                self.projects = reader;
                self.projects.set_focus(true);
                self.update_tasks();
            }
        }
    }
}
