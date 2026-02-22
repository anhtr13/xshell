pub mod builtin;
pub mod command;
pub mod helper;

use std::{
    fs::{OpenOptions, metadata},
    io::{self, Error},
    os::unix::fs::PermissionsExt,
    path::Path,
};

use crate::shell::command::Cmd;

pub fn check_is_excutable(name: &str) -> Result<String, String> {
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

pub fn parse_input(input: &str) -> io::Result<Vec<Cmd>> {
    if let Some(input) = shlex::split(input) {
        let mut cmds = Vec::new();
        let mut flag: u8 = 0;
        let mut name = "".to_string();
        let mut args = Vec::new();
        let mut stdout_file = None;
        let mut stderr_file = None;
        for arg in input {
            if flag == 0 {
                match arg.as_str() {
                    ">" | "1>" => flag = 1,
                    "2>" => flag = 2,
                    ">>" | "1>>" => flag = 3,
                    "2>>" => flag = 4,
                    "|" => {
                        if name.is_empty() {
                            return Err(Error::new(io::ErrorKind::InvalidInput, "parse error"));
                        }
                        cmds.push(Cmd {
                            name,
                            args,
                            stdout_file,
                            stderr_file,
                        });
                        name = "".to_string();
                        args = Vec::new();
                        stdout_file = None;
                        stderr_file = None;
                    }
                    _ => {
                        if name.is_empty() {
                            name = arg;
                        } else {
                            args.push(arg);
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
                        stdout_file = Some(f);
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
                        stderr_file = Some(f);
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
                        stdout_file = Some(f);
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
                        stderr_file = Some(f);
                        flag = 0;
                    }
                }
            }
        }
        if name.is_empty() {
            return Err(Error::new(io::ErrorKind::InvalidInput, "parse error"));
        } else {
            cmds.push(Cmd {
                name,
                args,
                stdout_file,
                stderr_file,
            });
        }
        return Ok(cmds);
    }
    Err(Error::new(io::ErrorKind::InvalidInput, "parse error"))
}
