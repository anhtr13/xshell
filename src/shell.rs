pub mod builtin;
pub mod command;
pub mod helper;
pub mod history;

use std::{
    fs::{OpenOptions, metadata},
    io::{self, Error},
    os::unix::fs::PermissionsExt,
    path::Path,
    str::FromStr,
};

use rustyline::Editor;

use crate::shell::{builtin::Builtin, command::Cmd, helper::InputHelper, history::History};

fn check_is_excutable(name: &str) -> Result<String, String> {
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
        return Err(format!("{name}: command not found"));
    };
    Err("Cannot get PATH".to_string())
}

fn parse_words(intput: &str) -> io::Result<Vec<String>> {
    enum State {
        Normal,
        NormalEscape,
        SingleQuote,
        DoubleQuote,
        DoubleQuoteEscape,
    }
    let mut args = Vec::new();
    let mut state = State::Normal;
    let mut word = String::new();
    for c in intput.trim().chars() {
        match state {
            State::Normal => match c {
                ' ' => {
                    if !word.is_empty() {
                        args.push(word);
                        word = String::new();
                    }
                }
                '\\' => state = State::NormalEscape,
                '\'' => state = State::SingleQuote,
                '\"' => state = State::DoubleQuote,
                _ => word.push(c),
            },
            State::NormalEscape => {
                word.push(c);
                state = State::Normal;
            }
            State::SingleQuote => match c {
                '\'' => state = State::Normal,
                _ => word.push(c),
            },
            State::DoubleQuote => match c {
                '\"' => state = State::Normal,
                '\\' => state = State::DoubleQuoteEscape,
                _ => word.push(c),
            },
            State::DoubleQuoteEscape => {
                match c {
                    '\\' => word.push('\\'),
                    '\'' => word.push('\''),
                    '\"' => word.push('\"'),
                    'n' => word.push('\n'),
                    't' => word.push('\t'),
                    '0' => word.push('\0'),
                    _ => word.push(c),
                };
                state = State::DoubleQuote;
            }
        }
    }
    if !word.is_empty() {
        args.push(word);
    }
    match state {
        State::Normal => Ok(args),
        _ => Err(Error::new(io::ErrorKind::InvalidInput, "parse error")),
    }
}

fn parse_input(input: &str) -> io::Result<Vec<Cmd>> {
    let words = parse_words(input)?;
    let mut cmds = Vec::new();
    let mut flag: u8 = 0;
    let mut name = "".to_string();
    let mut args = Vec::new();
    let mut stdout_file = None;
    let mut stderr_file = None;
    for arg in words {
        if flag == 0 {
            match arg.as_str() {
                ">" | "1>" => flag = 1,
                "2>" => flag = 2,
                ">>" | "1>>" => flag = 3,
                "2>>" => flag = 4,
                "|" => {
                    if name.is_empty() {
                        return Err(Error::new(io::ErrorKind::InvalidInput, "parse error"));
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
            }
        } else if flag == 1 {
            match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    return Err(Error::new(io::ErrorKind::InvalidInput, "parse error"));
                }
                _ => {
                    let f = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(&arg)?;
                    stdout_file = Some(f);
                    flag = 0;
                }
            }
        } else if flag == 2 {
            match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    return Err(Error::new(io::ErrorKind::InvalidInput, "parse error"));
                }
                _ => {
                    let f = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(&arg)?;
                    stderr_file = Some(f);
                    flag = 0;
                }
            }
        } else if flag == 3 {
            match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    return Err(Error::new(io::ErrorKind::InvalidInput, "parse error"));
                }
                _ => {
                    let f = OpenOptions::new().create(true).append(true).open(&arg)?;
                    stdout_file = Some(f);
                    flag = 0;
                }
            }
        } else if flag == 4 {
            match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    return Err(Error::new(io::ErrorKind::InvalidInput, "parse error"));
                }
                _ => {
                    let f = OpenOptions::new().create(true).append(true).open(&arg)?;
                    stderr_file = Some(f);
                    flag = 0;
                }
            }
        }
    }
    if name.is_empty() {
        return Err(Error::new(io::ErrorKind::InvalidInput, "parse error"));
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

pub fn run(rl: &mut Editor<InputHelper, History>) -> rustyline::Result<()> {
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
