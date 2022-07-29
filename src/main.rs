mod files;
mod ui;

use std::io;

use crossterm::event::{self, Event, KeyCode};
use files::{read_file_contents, parse_notes_file};
use tui::Terminal;
use ui::StatefulList;

pub struct App<'a> {
    // List of notes files
    items: StatefulList<&'a str>,
    // The content of the highlighted notes file
    select_item_content: StatefulList<&'a str>,
    // The notes directory
    working_directory: String,
}

fn main() -> Result<(), io::Error> {
    let mut terminal = ui::setup()?;
    let pwd = String::from(std::env::current_dir().unwrap().as_path().to_str().unwrap());
    let notes_dir = pwd + "/data/";
    let directory_files = files::get_directory_files(&notes_dir);
    let notes_files: Vec<&str> = directory_files.iter().map(|f| f.as_str()).collect();

    let first_file_contents = read_file_contents(&(notes_dir.clone() + notes_files[0]))
        .expect("Couldn't read first file");

    let mut app: App = App {
        items: StatefulList::with_items(notes_files),
        select_item_content: parse_notes_file(&notes_dir),
        working_directory: notes_dir,
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
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Up => {
                    app.items.next();
                    app.select_item_content = read_file_contents(
                        &(app.working_directory.clone() + app.items.get_current()),
                    )
                    .unwrap()
                }
                KeyCode::Down => {
                    app.items.previous();
                    app.select_item_content = read_file_contents(
                        &(app.working_directory.clone() + app.items.get_current()),
                    )
                    .unwrap()
                }
                _ => {}
            }
        }
    }
    Ok(())
}
