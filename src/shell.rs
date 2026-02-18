use std::{
    env::{current_dir, home_dir, set_current_dir},
    fmt::Display,
    fs::{File, OpenOptions, metadata},
    io::{self, Error},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{self, Child, ChildStdout, Command, Stdio},
    str::FromStr,
};

#[derive(Debug)]
pub struct BuiltinOutput {
    pub _status: u8,
    pub std_out: String,
    pub std_err: String,
}

#[derive(Debug)]
pub enum Builtin {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
}

impl Display for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cd => write!(f, "cd"),
            Self::Echo => write!(f, "exit"),
            Self::Exit => write!(f, "exit"),
            Self::Pwd => write!(f, "exit"),
            Self::Type => write!(f, "exit"),
        }
    }
}

impl FromStr for Builtin {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "exit" => Ok(Self::Exit),
            "echo" => Ok(Self::Echo),
            "type" => Ok(Self::Type),
            "pwd" => Ok(Self::Pwd),
            "cd" => Ok(Self::Cd),
            _ => Err("Not a builtin command"),
        }
    }
}

fn check_excutable(name: &str) -> Result<String, String> {
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
        } else if let Ok(path) = check_excutable(&args[0]) {
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

    pub fn run(&self, args: &[String]) -> BuiltinOutput {
        match self {
            Builtin::Cd => Builtin::run_cd(args),
            Builtin::Echo => Builtin::run_echo(args),
            Builtin::Exit => process::exit(0),
            Builtin::Pwd => Builtin::run_pwd(),
            Builtin::Type => Builtin::run_type(args),
        }
    }
}

#[derive(Debug)]
pub struct Cmd {
    pub name: String,
    pub args: Vec<String>,
    pub stdout_overwrite: Vec<File>,
    pub stderr_overwrite: Vec<File>,
    pub stdout_appends: Vec<File>,
    pub stderr_appends: Vec<File>,
}

impl Cmd {
    pub fn run(&self, prev_out: Option<ChildStdout>, is_last: bool) -> io::Result<Child> {
        if is_last {
            match prev_out {
                Some(pipe) => Command::new(&self.name)
                    .args(&self.args)
                    .stdin(Stdio::from(pipe))
                    .stdout(Stdio::inherit())
                    .spawn(),
                None => Command::new(&self.name)
                    .args(&self.args)
                    .stdout(Stdio::inherit())
                    .spawn(),
            }
        } else {
            Command::new(&self.name)
                .args(&self.args)
                .stdout(Stdio::piped())
                .spawn()
        }
    }
}

pub fn parse_input(input: &str) -> io::Result<Vec<Cmd>> {
    if let Some(input) = shlex::split(input) {
        let mut cmds = Vec::new();
        let mut flag: u8 = 0;
        let mut cmd_name = "".to_string();
        let mut cmd_args = Vec::new();
        let mut stdout_overwrite = Vec::new();
        let mut stderr_overwrite = Vec::new();
        let mut stdout_appends = Vec::new();
        let mut stderr_appends = Vec::new();
        for arg in input {
            if flag == 0 {
                match arg.as_str() {
                    ">" | "1>" => flag = 1,
                    "2>" => flag = 2,
                    ">>" | "1>>" => flag = 3,
                    "2>>" => flag = 4,
                    "|" => {
                        if cmd_name.is_empty() {
                            return Err(Error::new(io::ErrorKind::InvalidInput, "parse error"));
                        }
                        cmds.push(Cmd {
                            name: cmd_name,
                            args: cmd_args,
                            stdout_overwrite,
                            stderr_overwrite,
                            stdout_appends,
                            stderr_appends,
                        });
                        cmd_name = "".to_string();
                        cmd_args = Vec::new();
                        stdout_overwrite = Vec::new();
                        stderr_overwrite = Vec::new();
                        stdout_appends = Vec::new();
                        stderr_appends = Vec::new();
                    }
                    _ => {
                        if cmd_name.is_empty() {
                            cmd_name = arg;
                        } else {
                            cmd_args.push(arg);
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
                        stdout_overwrite.push(f);
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
                        stderr_overwrite.push(f);
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
                        stdout_appends.push(f);
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
                        stderr_appends.push(f);
                        flag = 0;
                    }
                }
            }
        }
        if cmd_name.is_empty() {
            return Err(Error::new(io::ErrorKind::InvalidInput, "parse error"));
        } else {
            cmds.push(Cmd {
                name: cmd_name,
                args: cmd_args,
                stdout_overwrite,
                stderr_overwrite,
                stdout_appends,
                stderr_appends,
            });
        }
        return Ok(cmds);
    }
    Err(Error::new(io::ErrorKind::InvalidInput, "parse error"))
}
