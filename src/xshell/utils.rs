use std::{
    fs::{OpenOptions, metadata},
    os::unix::fs::PermissionsExt,
    path::Path,
};

use crate::xshell::command::ShellCommand;

enum WordState {
    Normal,
    NormalEscape,
    SingleQuote,
    DoubleQuote,
    DoubleQuoteEscape,
}

pub fn words_from_input(input: &str) -> anyhow::Result<Vec<String>> {
    let mut args = Vec::new();
    let mut state = WordState::Normal;
    let mut word = String::new();
    for c in input.trim().chars() {
        match state {
            WordState::Normal => match c {
                ' ' => {
                    if !word.is_empty() {
                        args.push(word);
                        word = String::new();
                    }
                }
                '\\' => state = WordState::NormalEscape,
                '\'' => state = WordState::SingleQuote,
                '\"' => state = WordState::DoubleQuote,
                _ => word.push(c),
            },
            WordState::NormalEscape => {
                word.push(c);
                state = WordState::Normal;
            }
            WordState::SingleQuote => match c {
                '\'' => state = WordState::Normal,
                _ => word.push(c),
            },
            WordState::DoubleQuote => match c {
                '\"' => state = WordState::Normal,
                '\\' => state = WordState::DoubleQuoteEscape,
                _ => word.push(c),
            },
            WordState::DoubleQuoteEscape => {
                match c {
                    '\\' => word.push('\\'),
                    '\'' => word.push('\''),
                    '\"' => word.push('\"'),
                    'n' => word.push('\n'),
                    't' => word.push('\t'),
                    '0' => word.push('\0'),
                    _ => word.push(c),
                };
                state = WordState::DoubleQuote;
            }
        }
    }
    if !word.is_empty() {
        args.push(word);
    }
    match state {
        WordState::Normal => Ok(args),
        _ => anyhow::bail!("parse error: cannot parse word"),
    }
}

enum PipeState {
    Normal,
    RedirectingStdout,
    RedirectingStderr,
    AppendingStdout,
    AppendingStderr,
}

pub fn commands_from_input(input: &str) -> anyhow::Result<Vec<ShellCommand>> {
    let words = words_from_input(input)?;
    let mut cmds = Vec::new();
    let mut state = PipeState::Normal;
    let mut name = "".to_string();
    let mut args = Vec::new();
    let mut stdout_file = None;
    let mut stderr_file = None;

    for arg in words {
        match state {
            PipeState::Normal => match arg.as_str() {
                ">" | "1>" => state = PipeState::RedirectingStdout,
                "2>" => state = PipeState::RedirectingStderr,
                ">>" | "1>>" => state = PipeState::AppendingStdout,
                "2>>" => state = PipeState::AppendingStderr,
                "|" => {
                    if name.is_empty() {
                        anyhow::bail!("parse error: cannot parse input")
                    }
                    if let Some(last_arg) = args.last()
                        && last_arg == "&"
                    {
                        anyhow::bail!("parse error near `|`")
                    }
                    cmds.push(ShellCommand::new(
                        name,
                        args,
                        stdout_file,
                        stderr_file,
                        false,
                    ));
                    name = "".to_string();
                    args = Vec::new();
                    stdout_file = None;
                    stderr_file = None;
                }
                _ => {
                    if name.is_empty() {
                        name = arg;
                    } else {
                        args.push(arg);
                    }
                }
            },
            PipeState::RedirectingStdout => match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    anyhow::bail!("parse error: cannot parse input")
                }
                _ => {
                    let f = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(&arg)?;
                    stdout_file = Some(f);
                    state = PipeState::Normal;
                }
            },
            PipeState::RedirectingStderr => match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    anyhow::bail!("parse error: cannot parse input")
                }
                _ => {
                    let f = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(&arg)?;
                    stderr_file = Some(f);
                    state = PipeState::Normal;
                }
            },
            PipeState::AppendingStdout => match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    anyhow::bail!("parse error: cannot parse input")
                }
                _ => {
                    let f = OpenOptions::new().create(true).append(true).open(&arg)?;
                    stdout_file = Some(f);
                    state = PipeState::Normal;
                }
            },
            PipeState::AppendingStderr => match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    anyhow::bail!("parse error: cannot parse input")
                }
                _ => {
                    let f = OpenOptions::new().create(true).append(true).open(&arg)?;
                    stderr_file = Some(f);
                    state = PipeState::Normal;
                }
            },
        }
    }

    if name.is_empty() {
        anyhow::bail!("cannot parse input")
    } else {
        let is_background_job = args.pop_if(|x| x == "&").is_some();
        cmds.push(ShellCommand::new(
            name,
            args,
            stdout_file,
            stderr_file,
            is_background_job,
        ));
    }

    Ok(cmds)
}

pub fn check_command_excutable(cmd_name: &str) -> anyhow::Result<String> {
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let p = format!("{}/{}", dir.display(), cmd_name);
            let path = Path::new(&p);
            if path.is_file()
                && let Ok(metadata) = metadata(path)
                && let mode = metadata.permissions().mode()
                && (mode & 0o100 != 0 || mode & 0o010 != 0 || mode & 0o001 != 0)
            {
                return Ok(p);
            }
        }
        anyhow::bail!("{}: command not found", cmd_name)
    };
    anyhow::bail!("cannot get PATH")
}
