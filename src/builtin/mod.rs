pub mod cd;
pub mod echo;
pub mod history;
pub mod jobs;
pub mod pwd;
pub mod r#type;

use std::{fmt::Display, str::FromStr};

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
    Complete,
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
            Self::Complete => write!(f, "complete"),
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
            "complete" => Ok(Self::Complete),
            _ => anyhow::bail!("Not a builtin command"),
        }
    }
}
