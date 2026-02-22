use std::{
    env::{current_dir, home_dir, set_current_dir},
    fmt::Display,
    io::{self, PipeReader, Write},
    path::Path,
    process,
    str::FromStr,
};

use crate::shell::{check_is_excutable, command::Cmd, history::History};

#[derive(Debug, Default)]
pub struct BuiltinOutput {
    pub _status: u8,
    pub std_out: String,
    pub std_err: String,
}

impl BuiltinOutput {
    pub fn new(status: u8, std_out: String, std_err: String) -> Self {
        Self {
            _status: status,
            std_out,
            std_err,
        }
    }
}

#[derive(Debug)]
pub enum Builtin {
    Cd,
    Exit,
    Echo,
    History,
    Pwd,
    Type,
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
        }
    }
}

impl FromStr for Builtin {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cd" => Ok(Self::Cd),
            "echo" => Ok(Self::Echo),
            "exit" => Ok(Self::Exit),
            "history" => Ok(Self::History),
            "pwd" => Ok(Self::Pwd),
            "type" => Ok(Self::Type),
            _ => Err("Not a builtin command"),
        }
    }
}

impl Builtin {
    fn run_echo(args: &[String]) -> BuiltinOutput {
        BuiltinOutput::new(0, args.join(" "), "".to_string())
    }

    fn run_type(args: &[String]) -> BuiltinOutput {
        let (status, std_out, std_err) = if Builtin::from_str(&args[0]).is_ok() {
            (0, format!("{} is a shell builtin", args[0]), "".to_string())
        } else if let Ok(path) = check_is_excutable(&args[0]) {
            (0, format!("{} is {path}", args[0]), "".to_string())
        } else {
            (1, "".to_string(), format!("{}: not found", args[0]))
        };
        BuiltinOutput::new(status, std_out, std_err)
    }

    fn run_pwd() -> BuiltinOutput {
        match current_dir() {
            Ok(path) => BuiltinOutput::new(0, path.display().to_string(), "".to_string()),
            Err(e) => BuiltinOutput::new(1, "".to_string(), e.to_string()),
        }
    }

    fn run_cd(args: &[String]) -> BuiltinOutput {
        let mut home = String::new();
        if args.is_empty() || args[0].as_bytes().first() == Some(&b'~') {
            if let Some(h) = home_dir() {
                home = h.display().to_string();
            } else {
                return BuiltinOutput::new(
                    1,
                    "".to_string(),
                    "Impossible to get home dir".to_string(),
                );
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

    fn run_history(args: &[String], history: &mut History) -> BuiltinOutput {
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
            _status: 0,
            std_out: stdout,
            std_err: "".to_string(),
        }
    }

    pub fn run(&self, cmd: Cmd, history: &mut History, is_last: bool) -> Option<PipeReader> {
        let output = match self {
            Builtin::Cd => Self::run_cd(&cmd.args),
            Builtin::Echo => Self::run_echo(&cmd.args),
            Builtin::Exit => process::exit(0),
            Builtin::History => Self::run_history(&cmd.args, history),
            Builtin::Pwd => Self::run_pwd(),
            Builtin::Type => Self::run_type(&cmd.args),
        };
        if !output.std_err.is_empty() {
            if let Some(mut file) = cmd.stderr_file {
                writeln!(&mut file, "{}", output.std_err)
                    .unwrap_or_else(|e| eprintln!("Error: {e}"));
            } else {
                println!("{}", output.std_err);
            }
        }
        let mut pipeout = None;
        if !output.std_out.is_empty() {
            if let Some(mut file) = cmd.stdout_file {
                writeln!(&mut file, "{}", output.std_out)
                    .unwrap_or_else(|e| eprintln!("Error: {e}"));
            } else if !is_last {
                let (stdout_reader, mut stdout_writer) =
                    io::pipe().expect("Cannot create command pipeline");
                pipeout = Some(stdout_reader);
                writeln!(stdout_writer, "{}", output.std_out)
                    .unwrap_or_else(|e| eprintln!("Error: {e}"))
            } else {
                println!("{}", output.std_out);
            }
        }

        pipeout
    }
}
