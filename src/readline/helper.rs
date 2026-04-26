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

    fn register_completions(&self, line: &str) -> Result<Vec<String>> {
        let (lhs, word) = line.rsplit_once(' ').unwrap_or((line, ""));
        let (cmd, prev_word) = lhs.rsplit_once(' ').unwrap_or((line, ""));
        let Some(completer) = self.completers.get(cmd) else {
            anyhow::bail!("No completer registered");
        };
        let output = Command::new(completer)
            .args([cmd, word, prev_word])
            .output()?;

        let completions = String::from_utf8(output.stdout)?;
        let gap = if prev_word.is_empty() {
            " ".to_string()
        } else {
            format!(" {prev_word} ")
        };

        let candidates: Vec<_> = completions
            .trim_end_matches('\n')
            .split('\n')
            .map(|completion| format!("{cmd}{gap}{completion} "))
            .collect();

        Ok(candidates)
    }

    fn command_completions(prefix: &str) -> Vec<String> {
        let mut candidates = HashSet::new();

        let builtins = [
            "echo", "exit", "cd", "pwd", "type", "history", "jobs", "complete",
        ];

        for cmd in builtins.into_iter() {
            if cmd.starts_with(prefix) {
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
                    && name.starts_with(prefix)
                {
                    candidates.insert(name.to_string());
                }
            });
        }

        let mut candidates: Vec<String> = candidates.into_iter().collect();
        if candidates.len() == 1 {
            candidates[0].push(' ');
        }
        candidates
    }

    fn directory_completions(line: &str) -> Vec<String> {
        let mut candidates = Vec::new();
        let (lhs, path) = line.rsplit_once(' ').unwrap_or(("", line)); // lhs: left hand side
        let (dir, pattern) = path.rsplit_once('/').unwrap_or(("", path));
        let (prefix, dir) = if dir.is_empty() {
            (String::new(), ".")
        } else {
            (format!("{dir}/"), dir)
        };
        if let Ok(reader) = fs::read_dir(dir) {
            for entry in reader.filter_map(Result::ok) {
                let entry_name = entry.file_name().display().to_string();
                if entry_name.starts_with(pattern) {
                    if entry.path().is_dir() {
                        candidates.push(format!("{prefix}{entry_name}/"));
                    } else {
                        candidates.push(format!("{prefix}{entry_name}"));
                    }
                }
            }
        }
        if candidates.len() == 1 {
            let lhs = if lhs.is_empty() {
                String::new()
            } else {
                format!("{lhs} ")
            };
            candidates[0] = format!("{lhs}{}", candidates[0]);
            if !candidates[0].ends_with("/") {
                candidates[0].push(' ');
            }
        } else if candidates.len() >= 2 {
            let mut lcp = candidates[0].clone(); // lcp: longest common prefix
            for c in candidates.iter().skip(1) {
                while !lcp.is_empty() && !c.starts_with(&lcp) {
                    lcp.pop();
                }
                if lcp.is_empty() {
                    break;
                }
            }
            if !lcp.is_empty() && !line.ends_with(&lcp) {
                if lhs.is_empty() {
                    return vec![lcp];
                }
                return vec![format!("{lhs} {lcp}")];
            }
        }
        candidates
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
        if let Ok(candidates) = self.register_completions(line.trim_end())
            && !candidates.is_empty()
        {
            return Ok((0, candidates));
        }

        if pos == line.len() {
            let mut candidates = Self::command_completions(line);
            if candidates.is_empty() {
                candidates = Self::directory_completions(line);
            }
            if candidates.len() >= 2 {
                candidates.sort_unstable();
            }
            return Ok((0, candidates));
        }

        Ok((0, Vec::new()))
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
