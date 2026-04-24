use std::{
    collections::HashSet,
    fs::{self, metadata, read_dir},
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

    fn get_cmd_candidates(prefix: &str) -> Vec<String> {
        let mut candidates = HashSet::new();

        let builtins = ["echo", "exit", "cd", "pwd", "type", "history"];
        builtins.into_iter().for_each(|cmd| {
            if cmd.starts_with(prefix) {
                candidates.insert(cmd.to_string());
            }
        });

        if let Some(path) = std::env::var_os("PATH") {
            std::env::split_paths(&path)
                .map(|dir| read_dir(&dir))
                .filter_map(std::io::Result::ok)
                .for_each(|dir| {
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
                });
        }

        let mut candidates: Vec<String> = candidates.into_iter().collect();
        if candidates.len() == 1 {
            candidates[0].push(' ');
        }
        candidates
    }

    fn get_directory_completions(cmd: &str, dir_prefix: &str) -> Vec<String> {
        let mut candidates = Vec::new();
        let mut pre_arg = cmd.to_string();
        let paths: Vec<&str> = dir_prefix.split("/").collect();
        if paths.len() == 1 {
            let dir_prefix = paths[0];
            pre_arg.push(' ');
            if let Ok(reader) = fs::read_dir(".") {
                reader.filter_map(Result::ok).for_each(|entry| {
                    let dir_name = entry.file_name().display().to_string();
                    if dir_name.starts_with(dir_prefix) {
                        if entry.path().is_dir() {
                            candidates.push(format!("{dir_name}/"));
                        } else {
                            candidates.push(dir_name);
                        }
                    }
                });
            }
        } else if paths.len() >= 2 {
            let dir_prefix = paths[paths.len() - 1];
            let dir = paths[..paths.len() - 1].join("/");
            pre_arg = format!("{pre_arg} {dir}/");
            if let Ok(reader) = fs::read_dir(dir) {
                reader.filter_map(Result::ok).for_each(|entry| {
                    let dir_name = entry.file_name().display().to_string();
                    if dir_name.starts_with(dir_prefix) {
                        if entry.path().is_dir() {
                            candidates.push(format!("{dir_name}/"));
                        } else {
                            candidates.push(dir_name);
                        }
                    }
                });
            }
        }
        if candidates.len() == 1 {
            candidates[0] = format!("{pre_arg}{}", candidates[0]);
            if !candidates[0].ends_with("/") {
                candidates[0].push(' ');
            }
        } else if candidates.len() >= 2 {
            let mut lcp = candidates[0].clone();
            for c in candidates.iter().skip(1) {
                while !lcp.is_empty() && !c.starts_with(&lcp) {
                    lcp.pop();
                }
                if lcp.is_empty() {
                    break;
                }
            }
            if !lcp.is_empty() && lcp != dir_prefix {
                return vec![format!("{pre_arg}{lcp}")];
            }
        }
        candidates
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
        let mut args: Vec<&str> = line.split_whitespace().collect();
        let mut candidates = Vec::new();

        if pos == line.len() {
            if line.ends_with(" ") {
                args.push("");
            }
            if args.len() == 1 {
                candidates = Self::get_cmd_candidates(args[0]);
            } else if args.len() >= 2
                && let Some(prefix) = args.last()
                && !prefix.starts_with("-")
            {
                candidates =
                    Self::get_directory_completions(&args[..args.len() - 1].join(" "), prefix);
            }

            if candidates.len() >= 2 {
                candidates.sort_unstable();
            }

            return Ok((0, candidates));
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
