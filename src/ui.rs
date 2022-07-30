use std::io::{self, Stdout};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
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
    // Defines the left and right blocks
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(f.size());

    let items: Vec<ListItem> = app
        .files
        .items
        .iter()
        .map(|elem| ListItem::new(elem.clone()))
        .collect();

    let items = List::new(items)
        .block(Block::default().title("List").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(52, 235, 174))
                .fg(Color::Black),
        );


    let content: Vec<ListItem> = app
        .content
        .iter()
        .map(|elem| style_line(elem.to_string()))
        .collect();

    
    let content = List::new(content)
        .block(Block::default().title("List").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(52, 235, 174))
                .fg(Color::Black),
        );


    f.render_stateful_widget(items, chunks[0], &mut app.files.state);
    f.render_stateful_widget(content, chunks[1], &mut app.content_state.state);
}


fn style_line(line: String) -> ListItem<'static> {
    match line.chars().next() {
        Some('~') => {
            ListItem::new(drop_first(line))
                .style(Style::default().add_modifier(Modifier::CROSSED_OUT)).clone()
        }

        Some('*') => {
            ListItem::new(drop_first(line))
                .style(Style::default().add_modifier(Modifier::BOLD)).clone()
        }

        _ => ListItem::new(line)
                .style(Style::default()).clone()
    }
}

fn drop_first(s: String) -> String {
    let mut drop_styling = s.chars();
    drop_styling.next();
    drop_styling.as_str().to_string()
}

#[derive(Clone)]
pub struct StatefulList {
    state: ListState,
    items: Vec<String>,
}

impl StatefulList {
    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.state = ListState::default();
    }

    pub fn new(items: Vec<String>, set_first: bool) -> StatefulList {
        // TODO: this is probably a bad idea in case there aren't any files
        // in the chosen directory
        let mut state = ListState::default();

        if set_first {
            state.select(Some(0));
        }

        StatefulList { state, items }
    }

    pub fn get_current(&mut self) -> String {
        match self.state.selected() {
            Some(i) => self.items[i].clone(),
            None => self.items[0].clone(),
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

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
