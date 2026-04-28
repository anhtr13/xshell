use std::{
    collections::HashMap,
    env::{current_dir, home_dir, set_current_dir},
    fmt::Display,
    path::Path,
    str::FromStr,
};

use anyhow::Result;

use crate::{
    command::find_excutable,
    job::{Job, JobStatus},
    readline::history::History,
};

#[derive(Debug, PartialEq)]
pub enum Builtin {
    Cd,
    Exit,
    Echo,
    History,
    Pwd,
    Type,
    Jobs,
    Complete,
}

impl Display for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cd => write!(f, "cd"),
            Self::Echo => write!(f, "echo"),
            Self::Exit => write!(f, "exit"),
            Self::History => write!(f, "history"),
            Self::Pwd => write!(f, "pwd"),
            Self::Type => write!(f, "type"),
            Self::Jobs => write!(f, "jobs"),
            Self::Complete => write!(f, "complete"),
        }
    }
}

impl FromStr for Builtin {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cd" => Ok(Self::Cd),
            "echo" => Ok(Self::Echo),
            "exit" => Ok(Self::Exit),
            "history" => Ok(Self::History),
            "pwd" => Ok(Self::Pwd),
            "type" => Ok(Self::Type),
            "jobs" => Ok(Self::Jobs),
            "complete" => Ok(Self::Complete),
            _ => anyhow::bail!("Not a builtin command"),
        }
    }
}

pub fn cd(args: Vec<String>) -> Result<String> {
    let mut home = String::new();
    if args.is_empty() || args[0].as_bytes().first() == Some(&b'~') {
        if let Some(h) = home_dir() {
            home = h.display().to_string();
        } else {
            anyhow::bail!("Impossible to get home dir")
        }
    }
    let path_string = if args.is_empty() {
        home
    } else if args[0].as_bytes().first() == Some(&b'~') {
        format!("{}{}", home, &args[0][1..].to_string())
    } else {
        args[0].to_string()
    };
    match set_current_dir(Path::new(&path_string)) {
        Ok(_) => Ok(String::new()),
        Err(_) => anyhow::bail!("cd: {}: No such file or directory", path_string),
    }
}

pub fn echo(args: Vec<String>) -> Result<String> {
    Ok(args.join(" "))
}

pub fn complete(mut args: Vec<String>, completers: &mut HashMap<String, String>) -> Result<String> {
    if args.len() >= 2 {
        match args[0].as_str() {
            "-C" => {
                anyhow::ensure!(args.len() == 3);
                let command = args.pop().unwrap();
                let completer = args.pop().unwrap();
                completers.insert(command, completer);
            }
            "-p" => {
                anyhow::ensure!(args.len() == 2);
                if let Some(complete_path) = completers.get(&args[1]) {
                    return Ok(format!("complete -C '{}' {}", complete_path, args[1]));
                } else {
                    return Ok(format!(
                        "complete: {}: no completion specification",
                        args[1]
                    ));
                }
            }
            "-r" => {
                anyhow::ensure!(args.len() == 2);
                completers.remove(&args[1]);
            }
            _ => {}
        }
    }
    Ok(String::new())
}

pub fn history(args: Vec<String>, history: &mut History) -> Result<String> {
    let mut skip = 0;
    if !args.is_empty() {
        if let Ok(limit) = args[0].parse::<usize>() {
            skip = history.commands().len() - limit.min(history.commands().len());
        } else if args.len() >= 2 {
            if args[0] == "-r" {
                history.append_from_file(&args[1])?;
                return Ok(String::new());
            } else if args[0] == "-w" {
                history.write_to_file(&args[1])?;
                return Ok(String::new());
            } else if args[0] == "-a" {
                history.append_to_file(&args[1])?;
                return Ok(String::new());
            }
        }
    }
    let mut stdout = String::new();
    history
        .commands()
        .iter()
        .enumerate()
        .skip(skip)
        .for_each(|(i, cmd)| {
            if i + 1 == history.commands().len() {
                stdout.push_str(&format!("{:>5}  {}", i + 1, cmd));
            } else {
                stdout.push_str(&format!("{:>5}  {}\n", i + 1, cmd));
            }
        });
    Ok(stdout)
}

pub fn jobs(jobs: &[Job]) -> Result<String> {
    let mut output = Vec::new();
    for (i, job) in jobs.iter().enumerate() {
        let marker = if i + 1 == jobs.len() {
            "+"
        } else if i + 2 == jobs.len() {
            "-"
        } else {
            " "
        };
        let space = match job.status {
            JobStatus::Running => "                 ",
            JobStatus::Done => "                    ",
            JobStatus::Error => "                   ",
        };
        output.push(format!(
            "[{}]{}  {}{}{}",
            job.number, marker, job.status, space, job.command
        ));
    }
    Ok(output.join("\n"))
}

pub fn pwd() -> Result<String> {
    let dir = current_dir()?;
    Ok(dir.display().to_string())
}

pub fn r#type(args: Vec<String>) -> Result<String> {
    let ouput = if Builtin::from_str(&args[0]).is_ok() {
        format!("{} is a shell builtin", args[0])
    } else if let Some(ex_path) = find_excutable(&args[0]) {
        format!("{} is {}", args[0], ex_path)
    } else {
        anyhow::bail!("{}: not found", args[0])
    };
    Ok(ouput)
}
