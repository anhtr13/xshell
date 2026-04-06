use std::{
    env::{current_dir, home_dir, set_current_dir},
    fmt::Display,
    path::Path,
    str::FromStr,
};

use crate::xshell::{Job, JobStatus, history::History, utils::get_command_excutable};

#[derive(Debug, Default)]
pub struct BuiltinOutput {
    status: u8,
    std_out: String,
    std_err: String,
}

#[allow(unused)]
impl BuiltinOutput {
    pub fn new(status: u8, std_out: String, std_err: String) -> Self {
        Self {
            status,
            std_out,
            std_err,
        }
    }
    pub fn status(&self) -> u8 {
        self.status
    }
    pub fn std_out(&self) -> &str {
        &self.std_out
    }
    pub fn std_err(&self) -> &str {
        &self.std_err
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
    } else if let Ok(path) = get_command_excutable(&args[0]) {
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

pub fn run_job(mut jobs: Vec<Job>) -> BuiltinOutput {
    if jobs.is_empty() {
        return BuiltinOutput::new(0, "".to_string(), "".to_string());
    }

    jobs.sort_unstable_by_key(|x| x.created_at);

    let most_recent = jobs[jobs.len() - 1].pid;
    let second_most_recent = if jobs.len() == 1 {
        0
    } else {
        jobs[jobs.len() - 2].pid
    };

    jobs.sort_unstable_by_key(|x| x.number);

    let mut output = Vec::new();

    jobs.iter().for_each(|job| {
        let marker = if job.pid == most_recent {
            "+"
        } else if job.pid == second_most_recent {
            "-"
        } else {
            " "
        };
        let space = match job.status {
            JobStatus::Running => "                 ",
            JobStatus::Done => "                    ",
        };
        output.push(format!(
            "[{}]{}  {}{}{}",
            job.number, marker, job.status, space, job.command
        ));
    });

    BuiltinOutput::new(0, output.join("\n"), "".to_string())
}
