pub mod builtin;
pub mod command;
pub mod helper;
pub mod history;
mod utils;

use std::{
    io::{self, Write},
    process::Child,
    str::FromStr,
};

use rustyline::Editor;

use crate::xshell::{
    builtin::Builtin,
    helper::InputHelper,
    history::History,
    utils::{check_command_excutable, commands_from_input},
};

pub struct Shell<'a> {
    rl: &'a mut Editor<InputHelper, History>,
    background_jobs: Vec<Child>,
}

impl<'a> Shell<'a> {
    pub fn new(rl: &'a mut Editor<InputHelper, History>) -> Self {
        Self {
            rl,
            background_jobs: Vec::new(),
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            let input = self.rl.readline("$ ")?;
            let commands = commands_from_input(&input)?;
            let total_commands = commands.len();
            let mut command_io = None;

            for (idx, cmd) in commands.into_iter().enumerate() {
                let is_last = idx + 1 == total_commands;
                if let Ok(builtin) = Builtin::from_str(&cmd.name) {
                    let output = match builtin {
                        Builtin::Cd => builtin::run_cd(&cmd.args),
                        Builtin::Echo => builtin::run_echo(&cmd.args),
                        Builtin::Exit => return Ok(()),
                        Builtin::History => builtin::run_history(&cmd.args, self.rl.history_mut()),
                        Builtin::Pwd => builtin::run_pwd(),
                        Builtin::Type => builtin::run_type(&cmd.args),
                        Builtin::Jobs => builtin::run_job(),
                    };
                    if !output.std_err.is_empty() {
                        if let Some(mut file) = cmd.stderr_file {
                            writeln!(&mut file, "{}", output.std_err)?;
                        } else {
                            println!("{}", output.std_err);
                        }
                    }
                    command_io = None;
                    if !output.std_out.is_empty() {
                        if let Some(mut file) = cmd.stdout_file {
                            writeln!(&mut file, "{}", output.std_out)?;
                        } else if !is_last {
                            let (stdout_reader, mut stdout_writer) = io::pipe()?;
                            command_io = Some(stdout_reader);
                            writeln!(stdout_writer, "{}", output.std_out)?;
                        } else {
                            println!("{}", output.std_out);
                        }
                    }
                } else if let Err(e) = check_command_excutable(&cmd.name) {
                    eprintln!("{e}");
                    break;
                } else {
                    let is_background_job = cmd.is_background_job;
                    let (child, output) = cmd.run_external(command_io, is_last)?;
                    if is_background_job {
                        println!("[{}] {}", self.background_jobs.len() + 1, child.id());
                        self.background_jobs.push(child);
                        command_io = None;
                    } else {
                        command_io = output;
                    }
                }
            }
        }
    }
}
