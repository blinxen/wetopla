use std::env;
use std::fs::OpenOptions;
use std::io::Stdout;
use std::path::{Path, PathBuf};

use crate::event_loop::{Event, EventLoop};
use crate::project::ProjectContainer;
use crate::task::TaskContainer;
use crate::widgets::line_input::LineInput;
use crate::widgets::message_box::MessageBox;
use crate::widgets::{ContainerWidget, PopupWidget};
use crate::{terminal_utils, InputMode};
use crossterm::event::Event as CrosstermEvent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

const MAX_LOG_DURATION: u8 = 3;

// App holds the state of the application
pub struct TodoApp<'a> {
    // Current input mode
    input_mode: InputMode,
    // Container that will manage and display all projects
    projects: ProjectContainer,
    // Container that will manage and display all tasks of a project
    tasks: TaskContainer<'a>,
    // Bool that stores whether the user wants to quit
    // the application or not
    quit: bool,
    // Log message that is written at the bottom
    log_message: String,
    // Counter that keeps track of the duration of the current log message
    log_message_duration: u8,
    // When going in and out of the task editing mode
    // then the whole terminal needs to be redrawn
    // TODO: Find out why
    redraw: bool,
    // Whether the current state was modified
    dirty: bool,
    message_box: MessageBox,
    line_input: LineInput,
    // Event loop that controls draw and crossterm key events
    event_loop: EventLoop,
}

impl<'a> TodoApp<'a> {
    pub fn new() -> Self {
        Self {
            input_mode: InputMode::Normal,
            projects: ProjectContainer::new(true),
            tasks: TaskContainer::new(false),
            quit: false,
            log_message: String::new(),
            log_message_duration: 0,
            redraw: false,
            dirty: false,
            message_box: MessageBox::new(),
            line_input: LineInput::new(),
            event_loop: EventLoop::start(),
        }
    }

    pub async fn run(
        &mut self,
        mut terminal: Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), std::io::Error> {
        // Main loop
        loop {
            if self.redraw {
                terminal.resize(terminal.size()?)?;
                self.redraw = false;
            }

            match self.event_loop.event_rx.recv().await {
                Some(Event::Draw) => {
                    // Reset log message after 5 Draw events
                    // which equals to 5 seconds
                    if self.log_message_duration == MAX_LOG_DURATION {
                        self.log_message = String::new();
                    }
                    // Draw application with its widgets
                    self.draw(&mut terminal)?;
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
                    }
                    if self.input_mode == InputMode::Insert {
                        self.line_input.process_input(&key);
                    }
                    // Draw directly after processing input
                    self.draw(&mut terminal)?
                }
                _ => {}
            };

            if self.quit {
                break;
            }
        }

        Ok(())
    }

    fn draw(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), std::io::Error> {
        terminal.draw(|frame| {
            // Render main block
            frame.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Wetopla")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded),
                frame.size(),
            );
            // Create layout that will contain the body and the footer
            let outer_layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(97),
                        Constraint::Percentage(2),
                        Constraint::Percentage(1),
                    ]
                    .as_ref(),
                )
                .split(frame.size());
            // Create layout that will contain the main widgets
            let inner_layout = Layout::default()
                .direction(Direction::Horizontal)
                .horizontal_margin(1)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(outer_layout[0]);
            // Display projects
            frame.render_widget(self.projects.clone(), inner_layout[0]);
            // Display tasks
            frame.render_widget(self.tasks.clone(), inner_layout[1]);
            // Footer
            frame.render_widget(self.mode(), outer_layout[1]);
            frame.render_widget(Paragraph::new(self.log_message.clone()), outer_layout[2]);
            // Render helper widgets
            if self.input_mode == InputMode::Saving
                || self.input_mode == InputMode::Quitting
                || self.input_mode == InputMode::Deleting
            {
                self.message_box.render(frame);
            }

            if self.input_mode == InputMode::Insert {
                self.line_input.render(frame);
            }
        })?;

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
                        .set_question("Are you sure that you want to save?".to_string());
                }
            }
        }

        // Handle keys without modifier
        match self.input_mode {
            InputMode::Normal => {
                match key.code {
                    KeyCode::Char('q') => {
                        if self.dirty {
                            self.input_mode = InputMode::Quitting;
                            self.message_box.set_question(
                                "Do you want to save your changes before quitting?".to_string(),
                            );
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
                            .set_question("Are you sure that you want to delete?".to_string());
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
                    if !self.line_input.get_input().is_empty()
                        || self.line_input.get_input().len() > 3
                    {
                        self.input_mode = InputMode::Normal;
                        self.dirty = true;

                        if self.projects.is_focused() {
                            // Add project to projects list
                            self.projects.add_project(self.line_input.get_input());
                        } else {
                            // Add task to project and open editor to write the content of the task
                            self.projects
                                .current_project()
                                .unwrap()
                                .add_task(self.line_input.get_input());
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
        self.redraw = true;
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
        terminal_utils::restore_terminal()
            .expect("Error occured when trying to restore the previous state of the terminal!");
        // TODO: Think about adding some error handling here
        // Maybe display error in the log bar
        self.projects
            .current_project()
            .unwrap()
            .edit_task(task_index);
        terminal_utils::prepare_terminal()
            .expect("Error occured when trying to prepare the terminal for the application!");
        // Restart event loop after entering the application
        self.event_loop = EventLoop::start();
        self.update_tasks();
    }

    // Update tasks in task container
    fn update_tasks(&mut self) {
        self.tasks.set_project(self.projects.current_project());
    }

    fn mode(&self) -> Paragraph {
        let mode = if self.input_mode == InputMode::Normal {
            "NORMAL"
        } else if self.input_mode == InputMode::Insert {
            "INSERT"
        } else if self.input_mode == InputMode::Deleting {
            "DELETING"
        } else if self.input_mode == InputMode::Saving {
            "SAVING"
        } else if self.input_mode == InputMode::Quitting {
            "QUITTING"
        } else {
            "UNKNOWN MODE"
        };

        let style = if self.input_mode == InputMode::Normal {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else if self.input_mode == InputMode::Insert {
            Style::default().fg(Color::Black).bg(Color::Green)
        } else if self.input_mode == InputMode::Saving {
            Style::default().fg(Color::Black).bg(Color::Magenta)
        } else if self.input_mode == InputMode::Quitting {
            Style::default().fg(Color::Black).bg(Color::Gray)
        } else {
            Style::default().fg(Color::Black).bg(Color::Red)
        };
        Paragraph::new(mode)
            .style(style)
            .block(Block::default().borders(Borders::NONE))
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
