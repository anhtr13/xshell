use std::{
    collections::HashSet,
    fs::{metadata, read_dir},
    os::unix::fs::PermissionsExt,
};

use rustyline::{
    Helper, completion::Completer, highlight::Highlighter, hint::Hinter, validate::Validator,
};

pub struct InputHelper;

impl InputHelper {
    pub fn default() -> Self {
        InputHelper
    }

    fn get_candidates(word: &str) -> Vec<String> {
        let mut res = HashSet::new();
        if let Some(path) = std::env::var_os("PATH") {
            std::env::split_paths(&path)
                .map(|dir| read_dir(&dir))
                .filter_map(std::io::Result::ok)
                .for_each(|dir| {
                    dir.filter_map(std::io::Result::ok).for_each(|entry| {
                        let entry_path = entry.path();
                        if entry_path.is_file()
                            && let Some(name) = entry.file_name().to_str()
                            && !res.contains(name)
                            && let Ok(metadata) = metadata(&entry_path)
                            && let mode = metadata.permissions().mode()
                            && (mode & 0o100 != 0 || mode & 0o010 != 0 || mode & 0o001 != 0)
                            && name.starts_with(word)
                        {
                            res.insert(name.to_string());
                        }
                    });
                });
        }
        res.into_iter().collect()
    }
}

impl Completer for InputHelper {
    type Candidate = String;

    fn complete(
        &self, // FIXME should be `&mut self`
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let prefix = line;

        if pos == line.len() {
            match prefix {
                "ech" => return Ok((0, vec![String::from("echo ")])),
                "exi" => return Ok((0, vec![String::from("exit ")])),
                word => {
                    let mut candidates = Self::get_candidates(word);
                    if candidates.len() == 1 {
                        candidates[0].push(' ');
                    } else {
                        candidates.sort_unstable();
                    }
                    return Ok((0, candidates));
                }
            }
        }
        Ok((0, vec![String::from(line)]))
    }
    // fn update(
    //     &self,
    //     line: &mut rustyline::line_buffer::LineBuffer,
    //     start: usize,
    //     elected: &str,
    //     cl: &mut rustyline::Changeset,
    // ) {
    // }
}

impl Hinter for InputHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        None
    }
}

impl Validator for InputHelper {}

impl Highlighter for InputHelper {}

impl Helper for InputHelper {}
