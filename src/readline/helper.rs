use std::{
    collections::{HashMap, HashSet},
    fs::{self, metadata, read_dir},
    os::unix::fs::PermissionsExt,
    process::Command,
};

use anyhow::Result;
use rustyline::{
    Helper, completion::Completer, highlight::Highlighter, hint::Hinter, validate::Validator,
};

pub struct InputHelper {
    pub completers: HashMap<String, String>,
}

impl InputHelper {
    pub fn default() -> Self {
        InputHelper {
            completers: HashMap::new(),
        }
    }

    fn register_completions(&self, line: &str, pos: usize) -> Result<(usize, Vec<String>)> {
        let (lhs, word) = line.trim().rsplit_once(' ').unwrap_or((line.trim(), "")); // lhs: left hand side

        anyhow::ensure!(
            line.ends_with(' ') || !word.is_empty(),
            "Line must end with space or current word must not empty"
        );

        let (cmd, mut prev_word) = lhs
            .trim_end()
            .rsplit_once(' ')
            .unwrap_or((lhs.trim_end(), ""));
        if prev_word.is_empty() && !word.is_empty() {
            prev_word = cmd;
        }
        let args = [cmd, word, prev_word];
        let envs = HashMap::from([
            ("COMP_LINE", line.to_string()),
            ("COMP_POINT", pos.to_string()),
        ]);
        let Some(completer) = self.completers.get(cmd) else {
            anyhow::bail!("No completer registered");
        };

        let output = Command::new(completer).args(args).envs(envs).output()?;
        let completions = String::from_utf8(output.stdout)?;

        let mut candidates: Vec<_> = completions
            .trim_end_matches('\n')
            .split('\n')
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect();

        if candidates.len() == 1 {
            candidates[0].push(' ');
        }

        Ok((line.len() - word.len(), candidates))
    }

    fn command_completions(line: &str) -> Result<(usize, Vec<String>)> {
        anyhow::ensure!(!line.ends_with(' '));

        let (lhs, pattern) = line.rsplit_once(' ').unwrap_or(("", line));
        let mut candidates = HashSet::new();

        let builtins = [
            "echo", "exit", "cd", "pwd", "type", "history", "jobs", "complete",
        ];

        for cmd in builtins.into_iter() {
            if cmd.starts_with(pattern) {
                candidates.insert(cmd.to_string());
            }
        }

        let path = std::env::var_os("PATH").expect("PATH not found");
        for dir in std::env::split_paths(&path)
            .map(|dir| read_dir(&dir))
            .filter_map(std::io::Result::ok)
        {
            dir.filter_map(std::io::Result::ok).for_each(|entry| {
                let entry_path = entry.path();
                if entry_path.is_file()
                    && let Some(name) = entry.file_name().to_str()
                    && !candidates.contains(name)
                    && let Ok(metadata) = metadata(&entry_path)
                    && let mode = metadata.permissions().mode()
                    && (mode & 0o100 != 0 || mode & 0o010 != 0 || mode & 0o001 != 0)
                    && name.starts_with(pattern)
                {
                    candidates.insert(name.to_string());
                }
            });
        }

        let mut candidates: Vec<String> = candidates.into_iter().collect();
        if candidates.len() == 1 {
            candidates[0].push(' ');
        }

        Ok((lhs.len(), candidates))
    }

    fn directory_completions(line: &str) -> Result<(usize, Vec<String>)> {
        let (lhs, path) = line.rsplit_once(' ').unwrap_or(("", line));
        let (dir, pattern) = path.rsplit_once('/').unwrap_or(("", path));
        let (offset, dir) = match (lhs.len(), dir.len()) {
            (0, 0) => (0, "."),
            (x, 0) => (x + 1, "."),
            (0, y) => (y + 1, dir),
            (x, y) => (x + y + 2, dir),
        };
        let mut candidates = Vec::new();
        if let Ok(reader) = fs::read_dir(dir) {
            for entry in reader.filter_map(Result::ok) {
                let entry_name = entry.file_name().display().to_string();
                if entry_name.starts_with(pattern) {
                    if entry.path().is_dir() {
                        candidates.push(format!("{entry_name}/"));
                    } else {
                        candidates.push(entry_name);
                    }
                }
            }
        }
        if candidates.len() == 1 && !candidates[0].ends_with("/") {
            candidates[0].push(' ');
        }
        Ok((offset, candidates))
    }
}

impl Completer for InputHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let line = &line[..pos];
        if let Ok(completion) = self.register_completions(line, pos)
            && !completion.1.is_empty()
        {
            return Ok(completion);
        }
        let mut completion = Self::command_completions(line).unwrap_or((pos, Vec::new()));
        if completion.1.is_empty() {
            completion = Self::directory_completions(line).unwrap_or((pos, Vec::new()));
        }
        if completion.1.len() >= 2 {
            completion.1.sort_unstable();
        }

        Ok(completion)
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
