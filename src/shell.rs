use std::{
    env::{current_dir, home_dir, set_current_dir},
    fmt::Display,
    fs::{File, OpenOptions, metadata},
    io::{self, Error},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{self, Command},
    str::FromStr,
};

#[derive(Debug)]
pub struct Output {
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

impl Builtin {
    fn run_echo(args: &[String]) -> Output {
        Output {
            _status: 0,
            std_out: args.join(" "),
            std_err: "".to_string(),
        }
    }

    fn run_type(args: &[String]) -> Output {
        let (status, std_out, std_err) = if Builtin::from_str(&args[0]).is_ok() {
            (0, format!("{} is a shell builtin", args[0]), "".to_string())
        } else if let Ok(path) = Shell::check_excutable(&args[0]) {
            (0, format!("{} is {path}", args[0]), "".to_string())
        } else {
            (1, "".to_string(), format!("{}: not found", args[0]))
        };
        Output {
            _status: status,
            std_out,
            std_err,
        }
    }

    fn run_pwd() -> Output {
        match current_dir() {
            Ok(path) => Output {
                _status: 0,
                std_out: path.display().to_string(),
                std_err: "".to_string(),
            },
            Err(e) => Output {
                _status: 1,
                std_out: "".to_string(),
                std_err: e.to_string(),
            },
        }
    }

    fn run_cd(args: &[String]) -> Output {
        let mut home = String::new();
        if args.is_empty() || args[0].as_bytes().first() == Some(&b'~') {
            if let Some(h) = home_dir() {
                home = h.display().to_string();
            } else {
                return Output {
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
            Ok(_) => Output {
                _status: 0,
                std_out: "".to_string(),
                std_err: "".to_string(),
            },
            Err(_) => Output {
                _status: 1,
                std_out: "".to_string(),
                std_err: format!("cd: {}: No such file or directory", path_string),
            },
        }
    }

    pub fn run(&self, args: &[String]) -> Output {
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
pub struct Shell {
    pub cmd: String,
    pub args: Vec<String>,
    pub stdout_redirects: Vec<File>,
    pub stderr_redirects: Vec<File>,
    pub stdout_appends: Vec<File>,
    pub stderr_appends: Vec<File>,
}

impl Shell {
    pub fn parse_input(input: &str) -> io::Result<Self> {
        if let Some(mut cmd) = shlex::split(input) {
            let rest = cmd.split_off(1);
            let mut flag = 0;
            let mut args = Vec::new();
            let mut stdout_redirects = Vec::new();
            let mut stderr_redirects = Vec::new();
            let mut stdout_appends = Vec::new();
            let mut stderr_appends = Vec::new();
            for val in rest {
                if flag == 0 {
                    match val.as_str() {
                        ">" | "1>" => flag = 1,
                        "2>" => flag = 2,
                        ">>" | "1>>" => flag = 3,
                        "2>>" => flag = 4,
                        _ => args.push(val),
                    }
                } else if flag == 1 {
                    match val.as_str() {
                        ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" => {
                            return Err(Error::new(io::ErrorKind::InvalidInput, "parse error"));
                        }
                        _ => {
                            let f = OpenOptions::new()
                                .create(true)
                                .write(true)
                                .truncate(true)
                                .open(&val)?;
                            stdout_redirects.push(f);
                            flag = 0;
                        }
                    }
                } else if flag == 2 {
                    match val.as_str() {
                        ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" => {
                            return Err(Error::new(io::ErrorKind::InvalidInput, "parse error"));
                        }
                        _ => {
                            let f = OpenOptions::new()
                                .create(true)
                                .write(true)
                                .truncate(true)
                                .open(&val)?;
                            stderr_redirects.push(f);
                            flag = 0;
                        }
                    }
                } else if flag == 3 {
                    match val.as_str() {
                        ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" => {
                            return Err(Error::new(io::ErrorKind::InvalidInput, "parse error"));
                        }
                        _ => {
                            let f = OpenOptions::new().create(true).append(true).open(&val)?;
                            stdout_appends.push(f);
                            flag = 0;
                        }
                    }
                } else if flag == 4 {
                    match val.as_str() {
                        ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" => {
                            return Err(Error::new(io::ErrorKind::InvalidInput, "parse error"));
                        }
                        _ => {
                            let f = OpenOptions::new().create(true).append(true).open(&val)?;
                            stderr_appends.push(f);
                            flag = 0;
                        }
                    }
                }
            }
            return Ok(Shell {
                cmd: cmd.remove(0),
                args,
                stdout_redirects,
                stderr_redirects,
                stdout_appends,
                stderr_appends,
            });
        }
        Err(Error::new(io::ErrorKind::InvalidInput, "parse error"))
    }

    pub fn check_excutable(name: &str) -> Result<String, String> {
        if let Some(path) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path) {
                let p = format!("{}/{name}", dir.display());
                let path = Path::new(&p);
                if path.is_file() {
                    let mode = metadata(path).unwrap().permissions().mode();
                    if mode & 0o100 != 0 || mode & 0o010 != 0 || mode & 0o001 != 0 {
                        return Ok(p);
                    }
                }
            }
        };
        Err("Cannot get PATH".to_string())
    }

    fn run_executable(&self) -> Output {
        match Self::check_excutable(&self.cmd) {
            Ok(path) => match Command::new(path).args(&self.args).output() {
                Ok(output) => {
                    let mut std_err = output.stderr;
                    if let Some(c) = std_err.last()
                        && *c == b'\n'
                    {
                        std_err.pop();
                    }
                    let std_err = String::from_utf8(std_err).unwrap();
                    let mut std_out = output.stdout;
                    if let Some(c) = std_out.last()
                        && *c == b'\n'
                    {
                        std_out.pop();
                    }
                    let std_out = String::from_utf8(std_out).unwrap();
                    let status = if std_err.is_empty() { 0 } else { 1 };
                    Output {
                        _status: status,
                        std_out,
                        std_err,
                    }
                }
                Err(e) => Output {
                    _status: 1,
                    std_out: "".to_string(),
                    std_err: e.to_string(),
                },
            },
            Err(e) => Output {
                _status: 1,
                std_out: "".to_string(),
                std_err: e,
            },
        }
    }

    pub fn run(&self) -> Output {
        if let Ok(builtin) = Builtin::from_str(&self.cmd) {
            builtin.run(&self.args)
        } else {
            self.run_executable()
        }
    }
}
