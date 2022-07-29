use std::io::{self, Stdout};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

use crate::App;

pub fn setup() -> Result<Terminal<CrosstermBackend<Stdout>>, io::Error> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

pub fn teardown<B>(mut terminal: Terminal<B>) -> Result<(), io::Error>
where
    B: tui::backend::Backend,
    B: std::io::Write,
{
    // Returns terminal to original state
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

pub fn main_app<B>(f: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(f.size());

    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|elem| ListItem::new(*elem))
        .collect();

    let items = List::new(items)
        .block(Block::default().title("List").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(52, 235, 174))
                .fg(Color::Black),
        );

    let content = Paragraph::new(app.select_item_content.clone())
        .block(Block::default().title("Content").borders(Borders::ALL));

    f.render_stateful_widget(items, chunks[0], &mut app.items.state);
    f.render_widget(content, chunks[1]);
}

pub struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<'a, T> StatefulList<T>
where
    &'a str: From<T>,
    T: Clone,
    T: Copy,
{
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        // TODO: this is probably a bad idea in case there aren't any files
        // in the chosen directory
        let mut tmp_state = ListState::default();
        tmp_state.select(Some(0));
        StatefulList {
            state: tmp_state,
            items,
        }
    }

    pub fn get_current(&mut self) -> T {
        match self.state.selected() {
            Some(i) => self.items[i],
            None => self.items[0],
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
