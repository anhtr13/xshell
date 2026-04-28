use crate::command::ShellCommand;

use std::fs::OpenOptions;

enum ParseStateRedirecting {
    Normal,
    RedirectingStdout,
    RedirectingStderr,
    AppendingStdout,
    AppendingStderr,
}

pub fn commands_from_input(input: String) -> anyhow::Result<Vec<ShellCommand>> {
    let tokens = token_from_input(input)?;
    let mut cmds = Vec::new();
    let mut state = ParseStateRedirecting::Normal;
    let mut args = Vec::new();
    let mut name = String::from("");
    let mut stdout_file = None;
    let mut stderr_file = None;

    for arg in tokens {
        match state {
            ParseStateRedirecting::Normal => match arg.as_str() {
                ">" | "1>" => state = ParseStateRedirecting::RedirectingStdout,
                "2>" => state = ParseStateRedirecting::RedirectingStderr,
                ">>" | "1>>" => state = ParseStateRedirecting::AppendingStdout,
                "2>>" => state = ParseStateRedirecting::AppendingStderr,
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
            ParseStateRedirecting::RedirectingStdout => match arg.as_str() {
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
                    state = ParseStateRedirecting::Normal;
                }
            },
            ParseStateRedirecting::RedirectingStderr => match arg.as_str() {
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
                    state = ParseStateRedirecting::Normal;
                }
            },
            ParseStateRedirecting::AppendingStdout => match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    anyhow::bail!("parse error near `>`")
                }
                _ => {
                    let f = OpenOptions::new().create(true).append(true).open(&arg)?;
                    stdout_file = Some(f);
                    state = ParseStateRedirecting::Normal;
                }
            },
            ParseStateRedirecting::AppendingStderr => match arg.as_str() {
                ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" | "|" => {
                    anyhow::bail!("parse error near `>`")
                }
                _ => {
                    let f = OpenOptions::new().create(true).append(true).open(&arg)?;
                    stderr_file = Some(f);
                    state = ParseStateRedirecting::Normal;
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

enum ParseStateToken {
    Normal,
    NormalEscape,
    SingleQuote,
    DoubleQuote,
    DoubleQuoteEscape,
}

fn token_from_input(input: String) -> anyhow::Result<Vec<String>> {
    let mut args = Vec::new();
    let mut state = ParseStateToken::Normal;
    let mut token = String::new();
    for c in input.trim().chars() {
        match state {
            ParseStateToken::Normal => match c {
                ' ' => {
                    if !token.is_empty() {
                        args.push(token);
                        token = String::new();
                    }
                }
                '\\' => state = ParseStateToken::NormalEscape,
                '\'' => state = ParseStateToken::SingleQuote,
                '\"' => state = ParseStateToken::DoubleQuote,
                _ => token.push(c),
            },
            ParseStateToken::NormalEscape => {
                token.push(c);
                state = ParseStateToken::Normal;
            }
            ParseStateToken::SingleQuote => match c {
                '\'' => state = ParseStateToken::Normal,
                _ => token.push(c),
            },
            ParseStateToken::DoubleQuote => match c {
                '\"' => state = ParseStateToken::Normal,
                '\\' => state = ParseStateToken::DoubleQuoteEscape,
                _ => token.push(c),
            },
            ParseStateToken::DoubleQuoteEscape => {
                match c {
                    '\\' => token.push('\\'),
                    '\'' => token.push('\''),
                    '\"' => token.push('\"'),
                    'n' => token.push('\n'),
                    't' => token.push('\t'),
                    '0' => token.push('\0'),
                    _ => token.push(c),
                };
                state = ParseStateToken::DoubleQuote;
            }
        }
    }
    if !token.is_empty() {
        args.push(token);
    }
    match state {
        ParseStateToken::Normal => Ok(args),
        _ => anyhow::bail!("parse error: failed to parse token"),
    }
}
