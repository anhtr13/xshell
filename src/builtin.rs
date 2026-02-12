use std::{
    env::{current_dir, home_dir, set_current_dir},
    fmt::Display,
    path::Path,
    str::FromStr,
};

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

pub fn run_echo(args: &[String]) -> Option<String> {
    Some(args.join(" "))
}

pub fn run_type(args: &[String]) -> Option<String> {
    if Builtin::from_str(&args[0]).is_ok() {
        Some(format!("{} is a shell builtin", args[0]))
    } else if let Some(path) = crate::utils::find_excutable(&args[0]) {
        Some(format!("{} is {path}", args[0]))
    } else {
        Some(format!("{}: not found", args[0]))
    }
}

pub fn run_pwd() -> Option<String> {
    let path = current_dir().expect("Cannot get current directory.");
    Some(format!("{}", path.display()))
}

pub fn run_cd(args: &[String]) -> Option<String> {
    let path_string = if args.is_empty() {
        let home = home_dir().expect("Impossible to get home dir");
        home.display().to_string()
    } else {
        let mut input = args[0].to_string();
        if input.as_bytes().first() == Some(&b'~') {
            let home = home_dir().expect("Impossible to get home dir");
            input = format!("{}{}", home.display(), &input[1..]);
        }
        input
    };
    let path = Path::new(&path_string);
    match set_current_dir(path) {
        Ok(_) => None,
        Err(_) => Some(format!("{}: No such file or directory", &path_string)),
    }
}
