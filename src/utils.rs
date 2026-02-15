use std::{
    fs::{OpenOptions, metadata},
    io::{self, Error},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
};

use crate::{Output, shell::Cli};

pub fn parse_input(input: &str) -> io::Result<Cli> {
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
        return Ok(Cli {
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

pub fn find_excutable(name: &str) -> Option<String> {
    let path = std::env::var("PATH").expect("cannot get PATH");
    for dir in path.split(':') {
        let p = format!("{dir}/{name}");
        let path = Path::new(&p);
        if path.is_file() {
            let mode = metadata(path).unwrap().permissions().mode();
            if mode & 0o100 != 0 || mode & 0o010 != 0 || mode & 0o001 != 0 {
                return Some(p);
            }
        }
    }
    None
}

pub fn run_executable(path: &str, args: &Vec<String>) -> Output {
    match Command::new(path).args(args).output() {
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
    }
}
