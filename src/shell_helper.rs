use rustyline::{
    Helper, completion::Completer, highlight::Highlighter, hint::Hinter, validate::Validator,
};

pub struct ShellHelper;

impl ShellHelper {
    pub fn default() -> Self {
        ShellHelper
    }
}

impl Completer for ShellHelper {
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
                _ => {}
            }
        }
        Ok((0, vec![String::from("")]))
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

impl Hinter for ShellHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        None
    }
}

impl Validator for ShellHelper {}

impl Highlighter for ShellHelper {}

impl Helper for ShellHelper {}
