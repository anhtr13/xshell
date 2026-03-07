pub mod builtin;
pub mod command;
pub mod helper;
pub mod history;

use std::{
    fs::{OpenOptions, metadata},
    os::unix::fs::PermissionsExt,
    path::Path,
    str::FromStr,
};

use rustyline::Editor;

use crate::shell::{builtin::Builtin, command::Cmd, helper::InputHelper, history::History};

#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("{0}: command not found")]
    CmdNotFound(String),
    #[error("Not a builtin command")]
    NotBuiltin,
}

fn check_is_excutable(name: &str) -> Result<String, ShellError> {
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let p = format!("{}/{name}", dir.display());
            let path = Path::new(&p);
            if path.is_file()
                && let Ok(metadata) = metadata(path)
                && let mode = metadata.permissions().mode()
                && (mode & 0o100 != 0 || mode & 0o010 != 0 || mode & 0o001 != 0)
            {
                return Ok(p);
            }
        }
        return Err(ShellError::CmdNotFound(name.to_string()));
    };
    Err(ShellError::ParseError("cannot get PATH".to_string()))
}

enum ParseWordState {
    Normal,
    NormalEscape,
    SingleQuote,
    DoubleQuote,
    DoubleQuoteEscape,
}
fn parse_words(intput: &str) -> Result<Vec<String>, ShellError> {
    let mut args = Vec::new();
    let mut state = ParseWordState::Normal;
    let mut word = String::new();
    for c in intput.trim().chars() {
        match state {
            ParseWordState::Normal => match c {
                ' ' => {
                    if !word.is_empty() {
                        args.push(word);
                        word = String::new();
                    }
                }
                '\\' => state = ParseWordState::NormalEscape,
                '\'' => state = ParseWordState::SingleQuote,
                '\"' => state = ParseWordState::DoubleQuote,
                _ => word.push(c),
            },
            ParseWordState::NormalEscape => {
                word.push(c);
                state = ParseWordState::Normal;
            }
            ParseWordState::SingleQuote => match c {
                '\'' => state = ParseWordState::Normal,
                _ => word.push(c),
            },
            ParseWordState::DoubleQuote => match c {
                '\"' => state = ParseWordState::Normal,
                '\\' => state = ParseWordState::DoubleQuoteEscape,
                _ => word.push(c),
            },
            ParseWordState::DoubleQuoteEscape => {
                match c {
                    '\\' => word.push('\\'),
                    '\'' => word.push('\''),
                    '\"' => word.push('\"'),
                    'n' => word.push('\n'),
                    't' => word.push('\t'),
                    '0' => word.push('\0'),
                    _ => word.push(c),
                };
                state = ParseWordState::DoubleQuote;
            }
        }
    }
    if !word.is_empty() {
        args.push(word);
    }
    match state {
        ParseWordState::Normal => Ok(args),
        _ => Err(ShellError::ParseError("cannot parse words".to_string())),
    }
}

enum ParseInputState {
    Normal,
    RedirectingStdout,
    RedirectingStderr,
    AppendingStdout,
    AppendingStderr,
}
fn parse_input(input: &str) -> anyhow::Result<Vec<Cmd>> {
    let words = parse_words(input)?;
    let mut cmds = Vec::new();
    let mut state = ParseInputState::Normal;
    let mut name = "".to_string();
    let mut args = Vec::new();
    let mut stdout_file = None;
    let mut stderr_file = None;
    for arg in words {
        match state {
            ParseInputState::Normal => match arg.as_str() {
                ">" | "1>" => state = ParseInputState::RedirectingStdout,
                "2>" => state = ParseInputState::RedirectingStderr,
                ">>" | "1>>" => state = ParseInputState::AppendingStdout,
                "2>>" => state = ParseInputState::AppendingStderr,
                "|" => {
                    if name.is_empty() {
                        return Err(ShellError::ParseError("cannot parse input".to_string()).into());
                    }
                    cmds.push(Cmd {
                        name,
                        args,
                        stdout_file,
                        stderr_file,
                    });
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
            ParseInputState::RedirectingStdout => match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    return Err(ShellError::ParseError("cannot parse input".to_string()).into());
                }
                _ => {
                    let f = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(&arg)?;
                    stdout_file = Some(f);
                    state = ParseInputState::Normal;
                }
            },
            ParseInputState::RedirectingStderr => match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    return Err(ShellError::ParseError("cannot parse input".to_string()).into());
                }
                _ => {
                    let f = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(&arg)?;
                    stderr_file = Some(f);
                    state = ParseInputState::Normal;
                }
            },
            ParseInputState::AppendingStdout => match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    return Err(ShellError::ParseError("cannot parse input".to_string()).into());
                }
                _ => {
                    let f = OpenOptions::new().create(true).append(true).open(&arg)?;
                    stdout_file = Some(f);
                    state = ParseInputState::Normal;
                }
            },
            ParseInputState::AppendingStderr => match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    return Err(ShellError::ParseError("cannot parse input".to_string()).into());
                }
                _ => {
                    let f = OpenOptions::new().create(true).append(true).open(&arg)?;
                    stderr_file = Some(f);
                    state = ParseInputState::Normal;
                }
            },
        }
    }
    if name.is_empty() {
        return Err(ShellError::ParseError("cannot parse input".to_string()).into());
    } else {
        cmds.push(Cmd {
            name,
            args,
            stdout_file,
            stderr_file,
        });
    }
    Ok(cmds)
}

pub fn run(rl: &mut Editor<InputHelper, History>) -> anyhow::Result<()> {
    loop {
        let input = rl.readline("$ ")?;
        match parse_input(&input) {
            Ok(cmds) => {
                let mut cmd_io = None;
                let total_cmds = cmds.len();

                for (idx, cmd) in cmds.into_iter().enumerate() {
                    let is_last = idx + 1 == total_cmds;

                    if let Ok(builtin) = Builtin::from_str(&cmd.name) {
                        if builtin == Builtin::Exit {
                            return Ok(());
                        }
                        cmd_io = builtin.run(cmd, rl.history_mut(), is_last);
                    } else {
                        if let Err(e) = check_is_excutable(&cmd.name) {
                            eprintln!("{e}");
                            break;
                        }
                        cmd_io = cmd.run(cmd_io, is_last);
                    }
                }
            }
            Err(e) => eprintln!("{e}"),
        }
    }
}
