use anyhow::Result;

use crate::command::ShellCommand;

use std::{collections::HashMap, fs::OpenOptions};

enum RedirectingState {
    Normal,
    RedirectingStdout,
    RedirectingStderr,
    AppendingStdout,
    AppendingStderr,
}

pub fn commands_from_input(input: String) -> anyhow::Result<Vec<ShellCommand>> {
    let tokens = token_from_input(input)?;
    let mut cmds = Vec::new();
    let mut state = RedirectingState::Normal;
    let mut args = Vec::new();
    let mut name = String::from("");
    let mut stdout_file = None;
    let mut stderr_file = None;

    for arg in tokens {
        match state {
            RedirectingState::Normal => match arg.as_str() {
                ">" | "1>" => state = RedirectingState::RedirectingStdout,
                "2>" => state = RedirectingState::RedirectingStderr,
                ">>" | "1>>" => state = RedirectingState::AppendingStdout,
                "2>>" => state = RedirectingState::AppendingStderr,
                "|" => {
                    if name.is_empty() {
                        anyhow::bail!("parse error near `|`")
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
                    name = String::from("");
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
            RedirectingState::RedirectingStdout => match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    anyhow::bail!("parse error near `>`")
                }
                _ => {
                    let f = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(&arg)?;
                    stdout_file = Some(f);
                    state = RedirectingState::Normal;
                }
            },
            RedirectingState::RedirectingStderr => match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    anyhow::bail!("parse error near `>`")
                }
                _ => {
                    let f = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(&arg)?;
                    stderr_file = Some(f);
                    state = RedirectingState::Normal;
                }
            },
            RedirectingState::AppendingStdout => match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    anyhow::bail!("parse error near `>`")
                }
                _ => {
                    let f = OpenOptions::new().create(true).append(true).open(&arg)?;
                    stdout_file = Some(f);
                    state = RedirectingState::Normal;
                }
            },
            RedirectingState::AppendingStderr => match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    anyhow::bail!("parse error near `>`")
                }
                _ => {
                    let f = OpenOptions::new().create(true).append(true).open(&arg)?;
                    stderr_file = Some(f);
                    state = RedirectingState::Normal;
                }
            },
        }
    }

    if name.is_empty() {
        anyhow::bail!("parse error")
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

enum TokenState {
    Normal,
    NormalEscape,
    SingleQuote,
    DoubleQuote,
    DoubleQuoteEscape,
}

fn token_from_input(input: String) -> anyhow::Result<Vec<String>> {
    let mut args = Vec::new();
    let mut state = TokenState::Normal;
    let mut token = String::new();
    for c in input.trim().chars() {
        match state {
            TokenState::Normal => match c {
                ' ' => {
                    if !token.is_empty() {
                        args.push(token);
                        token = String::new();
                    }
                }
                '\\' => state = TokenState::NormalEscape,
                '\'' => state = TokenState::SingleQuote,
                '\"' => state = TokenState::DoubleQuote,
                _ => token.push(c),
            },
            TokenState::NormalEscape => {
                token.push(c);
                state = TokenState::Normal;
            }
            TokenState::SingleQuote => match c {
                '\'' => state = TokenState::Normal,
                _ => token.push(c),
            },
            TokenState::DoubleQuote => match c {
                '\"' => state = TokenState::Normal,
                '\\' => state = TokenState::DoubleQuoteEscape,
                _ => token.push(c),
            },
            TokenState::DoubleQuoteEscape => {
                match c {
                    '\\' => token.push('\\'),
                    '\'' => token.push('\''),
                    '\"' => token.push('\"'),
                    'n' => token.push('\n'),
                    't' => token.push('\t'),
                    '0' => token.push('\0'),
                    _ => token.push(c),
                };
                state = TokenState::DoubleQuote;
            }
        }
    }
    if !token.is_empty() {
        args.push(token);
    }
    match state {
        TokenState::Normal => Ok(args),
        _ => anyhow::bail!("parse error: failed to parse token"),
    }
}

#[derive(PartialEq)]
enum VariableState {
    Normal,
    Variable,
    VariableBrace,
}

pub fn args_expansion(
    args: Vec<String>,
    variables: &HashMap<String, String>,
) -> Result<Vec<String>> {
    let mut res = Vec::new();
    for arg in args {
        let mut state = VariableState::Normal;
        let mut var = String::new();
        let mut final_word = String::new();
        for c in arg.chars() {
            if c == '$' {
                match state {
                    VariableState::Normal => state = VariableState::Variable,
                    VariableState::Variable => {
                        if let Some(val) = variables.get(&var) {
                            final_word.push_str(val);
                        }
                        var = String::new();
                    }
                    VariableState::VariableBrace => {
                        anyhow::bail!("parse error: unexpected token near '$'")
                    }
                }
            } else if c == '{' {
                match state {
                    VariableState::Variable => state = VariableState::VariableBrace,
                    _ => anyhow::bail!("parse error: unexpected token near '{{'"),
                }
            } else if c == '}' {
                match state {
                    VariableState::VariableBrace => {
                        if let Some(val) = variables.get(&var) {
                            final_word.push_str(val);
                        }
                        var = String::new();
                        state = VariableState::Normal;
                    }
                    _ => anyhow::bail!("parse error: unexpected token near '}}'"),
                }
            } else {
                match state {
                    VariableState::Normal => final_word.push(c),
                    _ => var.push(c),
                }
            }
        }
        anyhow::ensure!(
            state != VariableState::VariableBrace,
            "parse error: unexpected token near '{{'"
        );
        if !var.is_empty()
            && let Some(val) = variables.get(&var)
        {
            final_word.push_str(val);
        }
        if !final_word.is_empty() {
            res.push(final_word);
        }
    }
    Ok(res)
}
