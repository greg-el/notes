use std::{
    fs::{read_dir, File},
    io::{BufRead, BufReader, Lines, Result, Write},
    path::Path,
};

// Get a list of the files in the notes directory,
// used to populate the left side of the notes UI
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

pub fn parse_notes_file(path: &str) -> Vec<String> {
    if let Ok(lines) = read_lines(path) {
        lines.flatten().map(|line| line.clone()).collect()
    } else {
        Vec::new()
    }
}

fn read_lines<P>(filename: P) -> Result<Lines<BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}

// Append a line to a notes file
pub fn edit_note(path: &str, line: &str, line_number: usize) {
    // Get the file as  a vector of lines
    let mut lines = parse_notes_file(path);

    // Replace the line at the given index with the new line
    lines[line_number] = line.to_string();

    // Write the new vector of lines to the file
    let mut file = File::create(path).unwrap();
    for line in lines {
        file.write_all(line.as_bytes()).unwrap();
        file.write_all(b"\n").unwrap();
    }
}
