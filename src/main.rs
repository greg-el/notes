mod files;
mod ui;

use std::io;

use crossterm::event::{self, Event, KeyCode};
use files::parse_notes_file;
use tui::Terminal;
use ui::StatefulList;

enum Window {
    FileList,
    Content,
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
}

impl App<'_> {
    pub fn update_content(&mut self) {
        let parsed_notes_file = parse_notes_file(&(self.working_directory.to_string() + &self.files.get_current()));
        self.content = parsed_notes_file.clone();
        self.content_state.set_items(self.content.clone());
    }
}

fn main() -> Result<(), io::Error> {
    let mut terminal = ui::setup()?;

    // Get the current program working directory
    let pwd = std::env::current_dir().unwrap().as_path().to_str().unwrap().to_string();

    // Get notes folder (hard-coded to /data at the moment)
    let notes_dir = pwd + "/data/";

    // Get a list of the file names in the folder
    let directory_files = files::get_directory_files(&notes_dir);

    // Parse the first file in the given notes folder
    let first_file_parsed = parse_notes_file(
        &(notes_dir.clone() + directory_files.first().expect("No files in given notes folder"))
    );

    // Create our App
    let mut app: App = App {
        files: StatefulList::new(directory_files, true),
        content: first_file_parsed.clone(),
        content_state: StatefulList::new(first_file_parsed, false),
        working_directory: &notes_dir,
        focused_window: Window::FileList,
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
                Window::Content => match key.code {
                    KeyCode::Char('q') => {
                        app.focused_window = Window::FileList;
                        app.content_state.unselect();
                    }
                    KeyCode::Char('j') => app.content_state.next(),
                    KeyCode::Char('k') => app.content_state.previous(),
                    _ => {}
                },
            }
        }
    }
    Ok(())
}
