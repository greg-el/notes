use std::io::{self, Stdout};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

use crate::{App, InputMode, Window};

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
    let items: Vec<ListItem> = app
        .files
        .items
        .iter()
        .map(|elem| ListItem::new(elem.clone()))
        .collect();

    let content: Vec<ListItem> = app
        .content
        .iter()
        .map(|elem| style_line(elem.to_string()))
        .collect();

    // Allows for styling the currently selected notes file (on the left) differently when editing it
    let (items, content) = match app.focused_window {
        Window::FileList => (
            // Filelist styling
            List::new(items)
                .block(Block::default().title("List").borders(Borders::ALL))
                .highlight_style(
                    Style::default()
                        .bg(Color::Rgb(52, 235, 174))
                        .fg(Color::Black),
                ),
            // Content styling
            List::new(content)
                .block(Block::default().title("List").borders(Borders::ALL))
                .highlight_style(
                    Style::default()
                        .bg(Color::Rgb(52, 235, 174))
                        .fg(Color::Black),
                ),
        ),
        Window::Content => (
            // Filelist styling
            List::new(items)
                .block(Block::default().title("List").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::Rgb(20, 20, 20)).fg(Color::Black)),
            // Content styling
            List::new(content)
                .block(Block::default().title("List").borders(Borders::ALL))
                .highlight_style(
                    Style::default()
                        .bg(Color::Rgb(52, 235, 174))
                        .fg(Color::Black),
                ),
        ),
    };

    match app.input_mode {
        InputMode::Normal => {
            // Defines the left and right blocks
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(f.size());

            f.render_stateful_widget(items, chunks[0], &mut app.files.state);
            f.render_stateful_widget(content, chunks[1], &mut app.content_state.state);
        }
        InputMode::Editing => {
            // Defines the left and right blocks
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(f.size());

            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                .split(chunks[1]);

            // Add on the input field on the bottom right if in input mode
            let input_widget = Paragraph::new(app.input.value())
                .block(Block::default().title("Input").borders(Borders::ALL))
                .style(Style::default());

            match app.input_mode {
                InputMode::Normal =>
                    // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
                    {}

                InputMode::Editing => {
                    // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                    f.set_cursor(
                        // Put cursor past the end of the input text
                        right_chunks[1].x
                            + (app.input.cursor() as u16).min(right_chunks[1].width)
                            + 1,
                        // Move one line down, from the border to the input line
                        right_chunks[1].y + 1,
                    )
                }
            }

            f.render_stateful_widget(items, chunks[0], &mut app.files.state);
            f.render_stateful_widget(content, right_chunks[0], &mut app.content_state.state);
            f.render_widget(input_widget, right_chunks[1]);
        }
    };

    match app.input_mode {
        InputMode::Normal => {}
        InputMode::Editing => {}
    }
}

fn style_line(line: String) -> ListItem<'static> {
    match line.chars().next() {
        Some('~') => ListItem::new(drop_first(line))
            .style(Style::default().add_modifier(Modifier::CROSSED_OUT))
            .clone(),

        Some('*') => ListItem::new(drop_first(line))
            .style(Style::default().add_modifier(Modifier::BOLD))
            .clone(),

        _ => ListItem::new(line).style(Style::default()).clone(),
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

    pub fn set_items_with_index(&mut self, items: Vec<String>, index: usize) {
        self.items = items;
        let mut state = ListState::default();
        state.select(Some(index));
        self.state = state;
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

    pub fn get_current_index(&self) -> Option<usize> {
        self.state.selected()
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
