use std::{
    env::{current_dir, home_dir, set_current_dir},
    fmt::Display,
    io::{self, PipeReader, Write},
    path::Path,
    process,
    str::FromStr,
};

use crate::shell::{check_is_excutable, command::Cmd};

#[derive(Debug)]
pub struct BuiltinOutput {
    pub _status: u8,
    pub std_out: String,
    pub std_err: String,
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
        BuiltinOutput {
            _status: 0,
            std_out: args.join(" "),
            std_err: "".to_string(),
        }
    }

    fn run_type(args: &[String]) -> BuiltinOutput {
        let (status, std_out, std_err) = if Builtin::from_str(&args[0]).is_ok() {
            (0, format!("{} is a shell builtin", args[0]), "".to_string())
        } else if let Ok(path) = check_is_excutable(&args[0]) {
            (0, format!("{} is {path}", args[0]), "".to_string())
        } else {
            (1, "".to_string(), format!("{}: not found", args[0]))
        };
        BuiltinOutput {
            _status: status,
            std_out,
            std_err,
        }
    }

    fn run_pwd() -> BuiltinOutput {
        match current_dir() {
            Ok(path) => BuiltinOutput {
                _status: 0,
                std_out: path.display().to_string(),
                std_err: "".to_string(),
            },
            Err(e) => BuiltinOutput {
                _status: 1,
                std_out: "".to_string(),
                std_err: e.to_string(),
            },
        }
    }

    fn run_cd(args: &[String]) -> BuiltinOutput {
        let mut home = String::new();
        if args.is_empty() || args[0].as_bytes().first() == Some(&b'~') {
            if let Some(h) = home_dir() {
                home = h.display().to_string();
            } else {
                return BuiltinOutput {
                    _status: 1,
                    std_out: "".to_string(),
                    std_err: "Impossible to get home dir".to_string(),
                };
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
            Ok(_) => BuiltinOutput {
                _status: 0,
                std_out: "".to_string(),
                std_err: "".to_string(),
            },
            Err(_) => BuiltinOutput {
                _status: 1,
                std_out: "".to_string(),
                std_err: format!("cd: {}: No such file or directory", path_string),
            },
        }
    }

    fn run_history(history: &[String]) -> BuiltinOutput {
        let mut stdout = String::new();
        history.iter().enumerate().for_each(|(i, cmd)| {
            if i + 1 == history.len() {
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

    pub fn run(&self, cmd: Cmd, history: &mut Vec<String>, is_last: bool) -> Option<PipeReader> {
        let output = match self {
            Builtin::Cd => Self::run_cd(&cmd.args),
            Builtin::Echo => Self::run_echo(&cmd.args),
            Builtin::Exit => process::exit(0),
            Builtin::History => Self::run_history(history),
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
                let (stdout_reader, mut stdout_writer) = io::pipe().expect("Cannot create pipe");
                pipeout = Some(stdout_reader);
                writeln!(stdout_writer, "{}", output.std_out)
                    .unwrap_or_else(|e| eprintln!("Error: {e}"))
            } else {
                println!("{}", output.std_out);
            }
        }

        history.push(format!("{} {}", cmd.name, cmd.args.join(" ")));

        pipeout
    }
}
