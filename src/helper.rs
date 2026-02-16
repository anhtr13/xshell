use std::fs::read_dir;

use rustyline::{
    Helper, completion::Completer, highlight::Highlighter, hint::Hinter, validate::Validator,
};

pub struct InputHelper;

impl InputHelper {
    pub fn default() -> Self {
        InputHelper
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
                pre => {
                    if let Some(path) = std::env::var_os("PATH") {
                        for dir in std::env::split_paths(&path) {
                            if let Ok(read_dir) = read_dir(&dir) {
                                for entry in read_dir.filter_map(std::io::Result::ok) {
                                    if let Some(name) = entry.file_name().to_str()
                                        && name.starts_with(pre)
                                    {
                                        return Ok((0, vec![name.to_string()]));
                                    }
                                }
                            }
                        }
                    }
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
