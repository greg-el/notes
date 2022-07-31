mod files;
mod ui;

use crossterm::event::{self, Event, KeyCode};
use files::parse_notes_file;
use std::io;
use tui::Terminal;
use tui_input::backend::crossterm as input_backend;
use tui_input::Input;
use ui::StatefulList;

enum Window {
    FileList,
    Content,
}

enum InputMode {
    Normal,
    Editing,
}

pub struct App<'a> {
    // List of notes files
    files: StatefulList,
    // The state of the highlighted notes file
    content_state: StatefulList,
    // The currently selected file
    content: Vec<String>,
    // The notes directory
    working_directory: &'a str,
    focused_window: Window,
    input_mode: InputMode,
    // selected_line: String,
    input: Input,
}

impl App<'_> {
    pub fn get_current_file_path(&mut self) -> String {
        format!("{}/{}", self.working_directory, self.files.get_current())
    }

    pub fn update_content(&mut self) {
        let parsed_notes_file =
            parse_notes_file(&(self.working_directory.to_string() + &self.files.get_current()));
        self.content = parsed_notes_file.clone();
        self.content_state.set_items(self.content.clone());
    }

    // Useful for exiting input mode, keeping the selected note line highlighted
    pub fn update_content_with_selected(&mut self, index: usize) {
        let parsed_notes_file =
            parse_notes_file(&(self.working_directory.to_string() + &self.files.get_current()));
        self.content = parsed_notes_file.clone();
        self.content_state
            .set_items_with_index(self.content.clone(), index);
    }
}

fn main() -> Result<(), io::Error> {
    let mut terminal = ui::setup()?;

    // Get the current program working directory
    let pwd = std::env::current_dir()
        .unwrap()
        .as_path()
        .to_str()
        .unwrap()
        .to_string();

    // Get notes folder (hard-coded to /data at the moment)
    let notes_dir = pwd + "/data/";

    // Get a list of the file names in the folder
    let directory_files = files::get_directory_files(&notes_dir);

    // Parse the first file in the given notes folder
    let first_file_parsed = parse_notes_file(
        &(notes_dir.clone()
            + directory_files
                .first()
                .expect("No files in given notes folder")),
    );

    // Create our App
    let mut app: App = App {
        files: StatefulList::new(directory_files, true),
        content: first_file_parsed.clone(),
        content_state: StatefulList::new(first_file_parsed, false),
        working_directory: &notes_dir,
        focused_window: Window::FileList,
        input_mode: InputMode::Normal,
        // selected_line: String::new(),
        input: Input::default(),
    };

    run_app(&mut terminal, &mut app)?;
    ui::teardown(terminal)
}

fn run_app<B>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), io::Error>
where
    B: tui::backend::Backend,
    B: std::io::Write,
{
    loop {
        terminal.draw(|f| ui::main_app(f, app))?;

        if let Event::Key(key) = event::read()? {
            match app.focused_window {
                Window::FileList => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('e') => {
                        app.focused_window = Window::Content;
                        app.content_state.next();
                    }
                    KeyCode::Char('j') => {
                        app.files.next();
                        app.update_content();
                    }
                    KeyCode::Char('k') => {
                        app.files.previous();
                        app.update_content();
                    }
                    _ => {}
                },
                Window::Content => match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => {
                            app.focused_window = Window::FileList;
                            app.content_state.unselect();
                        }
                        KeyCode::Char('j') => app.content_state.next(),
                        KeyCode::Char('k') => app.content_state.previous(),
                        KeyCode::Char('e') => {
                            app.input = Input::new(app.content_state.get_current());
                            app.input_mode = InputMode::Editing;
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            let curr_index = app.content_state.get_current_index().unwrap();
                            files::edit_note(
                                &app.get_current_file_path(),
                                app.input.value(),
                                curr_index,
                            );
                            app.update_content_with_selected(curr_index);
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        // handles tui-input's business
                        _ => {
                            input_backend::to_input_request(Event::Key(key))
                                .and_then(|req| app.input.handle(req));
                        }
                    },
                },
            }
        }
    }
    Ok(())
}
