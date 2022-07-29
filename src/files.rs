use std::{
    fs::{self, read_dir, File},
    io::{self, BufRead},
    path::Path,
};

use tui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{List, ListItem},
};

pub fn get_directory_files(dir: &str) -> Vec<String> {
    let paths = read_dir(dir).unwrap();

    paths
        .filter_map(|path| {
            path.ok().and_then(|p| {
                p.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(String::from)
            })
        })
        .collect()
}

pub fn read_file_contents(path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}

pub fn parse_notes_file(path: &str) -> Vec<ListItem> {
    if let Ok(lines) = read_lines(path) {
        lines
            .flatten()
            .map(|line| ListItem::new(line.clone()).style(style_line(line)))
            .collect()
    } else {
        Vec::new()
    }
}

fn style_line(line: String) -> Style {
    match line.chars().next() {
        Some('~') => Style::default().add_modifier(Modifier::CROSSED_OUT),
        Some('*') => Style::default().add_modifier(Modifier::BOLD),
        _ => Style::default(),
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
