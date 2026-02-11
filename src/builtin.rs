use std::{
    env::{current_dir, home_dir, set_current_dir},
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

pub fn run_echo(args: &[String]) {
    let output = args.join(" ");
    println!("{output}");
}

pub fn run_type(args: &[String]) {
    if Builtin::from_str(&args[0]).is_ok() {
        println!("{} is a shell builtin", args[0]);
    } else if let Some(path) = crate::utils::find_excutable(&args[0]) {
        println!("{} is {path}", args[0])
    } else {
        println!("{}: not found", args[0]);
    }
}

pub fn run_pwd() {
    let path = current_dir().expect("Cannot get current directory.");
    println!("{}", path.display());
}

pub fn run_cd(args: &[String]) {
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
    if path.is_dir() {
        set_current_dir(path).unwrap_or_else(|_| {
            panic!("{}: No such file or directory", &path_string);
        })
    } else {
        eprintln!("{}: No such file or directory", &path_string);
    }
}
