use std::{
    collections::HashMap,
    env::{current_dir, home_dir, set_current_dir},
    fmt::Display,
    path::Path,
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::xshell::{Job, history::History, utils::check_command_excutable};

#[allow(unused)]
#[derive(Debug, Default)]
pub struct BuiltinOutput {
    pub status: u8,
    pub std_out: String,
    pub std_err: String,
}

impl BuiltinOutput {
    pub fn new(status: u8, std_out: String, std_err: String) -> Self {
        Self {
            status,
            std_out,
            std_err,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Builtin {
    Cd,
    Exit,
    Echo,
    History,
    Pwd,
    Type,
    Jobs,
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
            _ => anyhow::bail!("Not a builtin command"),
        }
    }
}

pub fn run_echo(args: &[String]) -> BuiltinOutput {
    BuiltinOutput::new(0, args.join(" "), "".to_string())
}

pub fn run_type(args: &[String]) -> BuiltinOutput {
    let (status, std_out, std_err) = if Builtin::from_str(&args[0]).is_ok() {
        (0, format!("{} is a shell builtin", args[0]), "".to_string())
    } else if let Ok(path) = check_command_excutable(&args[0]) {
        (0, format!("{} is {path}", args[0]), "".to_string())
    } else {
        (1, "".to_string(), format!("{}: not found", args[0]))
    };
    BuiltinOutput::new(status, std_out, std_err)
}

pub fn run_pwd() -> BuiltinOutput {
    match current_dir() {
        Ok(path) => BuiltinOutput::new(0, path.display().to_string(), "".to_string()),
        Err(e) => BuiltinOutput::new(1, "".to_string(), e.to_string()),
    }
}

pub fn run_cd(args: &[String]) -> BuiltinOutput {
    let mut home = String::new();
    if args.is_empty() || args[0].as_bytes().first() == Some(&b'~') {
        if let Some(h) = home_dir() {
            home = h.display().to_string();
        } else {
            return BuiltinOutput::new(1, "".to_string(), "Impossible to get home dir".to_string());
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
        Ok(_) => BuiltinOutput::default(),
        Err(_) => BuiltinOutput::new(
            1,
            "".to_string(),
            format!("cd: {}: No such file or directory", path_string),
        ),
    }
}

pub fn run_history(args: &[String], history: &mut History) -> BuiltinOutput {
    let mut skip = 0;
    if !args.is_empty() {
        if let Ok(limit) = args[0].parse::<usize>() {
            skip = history.commands.len() - limit.min(history.commands.len());
        } else if args.len() >= 2 {
            if args[0] == "-r" {
                match history.append_from_file(&args[1]) {
                    Ok(_) => {
                        return BuiltinOutput::default();
                    }
                    Err(e) => {
                        return BuiltinOutput::new(1, "".to_string(), e.to_string());
                    }
                }
            } else if args[0] == "-w" {
                match history.write_to_file(&args[1]) {
                    Ok(_) => {
                        return BuiltinOutput::default();
                    }
                    Err(e) => {
                        return BuiltinOutput::new(1, "".to_string(), e.to_string());
                    }
                }
            } else if args[0] == "-a" {
                match history.append_to_file(&args[1]) {
                    Ok(_) => {
                        return BuiltinOutput::default();
                    }
                    Err(e) => {
                        return BuiltinOutput::new(1, "".to_string(), e.to_string());
                    }
                }
            }
        }
    }
    let mut stdout = String::new();
    history
        .commands
        .iter()
        .enumerate()
        .skip(skip)
        .for_each(|(i, cmd)| {
            if i + 1 == history.commands.len() {
                stdout.push_str(&format!("{:>5}  {}", i + 1, cmd));
            } else {
                stdout.push_str(&format!("{:>5}  {}\n", i + 1, cmd));
            }
        });
    BuiltinOutput {
        status: 0,
        std_out: stdout,
        std_err: "".to_string(),
    }
}

pub fn run_job(jobs: Arc<Mutex<HashMap<u32, Job>>>) -> BuiltinOutput {
    let guard = jobs.lock().unwrap();

    let mut jobs: Vec<&Job> = guard.values().clone().collect();
    jobs.sort_unstable_by_key(|x| x.job_id);

    let mut output = Vec::new();
    for job in jobs {
        output.push(format!(
            "[{}]+  Running                 {}",
            job.job_id, job.command
        ));
    }

    BuiltinOutput::new(0, output.join("\n"), "".to_string())
}
