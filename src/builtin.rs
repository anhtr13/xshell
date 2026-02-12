use std::{
    env::{current_dir, home_dir, set_current_dir},
    fmt::Display,
    path::Path,
    str::FromStr,
};

use crate::CmdOutput;

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

pub fn run_echo(args: &[String]) -> CmdOutput {
    CmdOutput {
        status: 0,
        std_out: args.join(" "),
        std_err: "".to_string(),
    }
}

pub fn run_type(args: &[String]) -> CmdOutput {
    let (status, std_out, std_err) = if Builtin::from_str(&args[0]).is_ok() {
        (0, format!("{} is a shell builtin", args[0]), "".to_string())
    } else if let Some(path) = crate::utils::find_excutable(&args[0]) {
        (0, format!("{} is {path}", args[0]), "".to_string())
    } else {
        (1, "".to_string(), format!("{}: not found", args[0]))
    };
    CmdOutput {
        status,
        std_out,
        std_err,
    }
}

pub fn run_pwd() -> CmdOutput {
    match current_dir() {
        Ok(path) => CmdOutput {
            status: 0,
            std_out: path.display().to_string(),
            std_err: "".to_string(),
        },
        Err(e) => CmdOutput {
            status: 1,
            std_out: "".to_string(),
            std_err: e.to_string(),
        },
    }
}

pub fn run_cd(args: &[String]) -> CmdOutput {
    let mut home = String::new();
    if args.is_empty() || args[0].as_bytes().first() == Some(&b'~') {
        if let Some(h) = home_dir() {
            home = h.display().to_string();
        } else {
            return CmdOutput {
                status: 1,
                std_out: "".to_string(),
                std_err: "Impossible to get home dir".to_string(),
            };
        }
    }
    let path_string = if args.is_empty() {
        home
    } else if args[0].as_bytes().first() == Some(&b'~') {
        format!("{}{}", home, args[0][1..].to_string())
    } else {
        args[0].to_string()
    };
    match set_current_dir(Path::new(&path_string)) {
        Ok(_) => CmdOutput {
            status: 0,
            std_out: "".to_string(),
            std_err: "".to_string(),
        },
        Err(e) => CmdOutput {
            status: 1,
            std_out: "".to_string(),
            std_err: e.to_string(),
        },
    }
}
