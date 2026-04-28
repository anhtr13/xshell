use std::{
    io::{self, Write},
    str::FromStr,
};

use rustyline::Editor;

use crate::{
    builtin::{self, Builtin},
    command::find_excutable,
    job::Jobs,
    parser,
    readline::{helper::Helper, history::History},
};

pub struct Shell<'a> {
    editor: &'a mut Editor<Helper, History>,
    jobs: Jobs,
}

impl<'a> Shell<'a> {
    pub fn new(editor: &'a mut Editor<Helper, History>) -> Self {
        let jobs = Jobs::new();
        Self { editor, jobs }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            let input = self.editor.readline("$ ")?;
            let commands = parser::commands_from_input(input)?;
            let total_commands = commands.len();
            let mut command_io = None;

            let mut has_job_builtin = false;

            self.jobs.update_status();

            for (idx, cmd) in commands.into_iter().enumerate() {
                let is_last = idx + 1 == total_commands;
                if let Ok(builtin) = Builtin::from_str(&cmd.name) {
                    let output = match builtin {
                        Builtin::Cd => builtin::cd(cmd.args),
                        Builtin::Echo => builtin::echo(cmd.args),
                        Builtin::History => builtin::history(cmd.args, self.editor.history_mut()),
                        Builtin::Pwd => builtin::pwd(),
                        Builtin::Type => builtin::r#type(cmd.args),
                        Builtin::Jobs => {
                            has_job_builtin = true;
                            builtin::jobs(self.jobs.value())
                        }
                        Builtin::Complete => builtin::complete(
                            cmd.args,
                            &mut self.editor.helper_mut().unwrap().completers,
                        ),
                        Builtin::Exit => return Ok(()),
                    };
                    command_io = None;
                    match output {
                        Ok(std_out) => {
                            if !std_out.is_empty() {
                                if let Some(mut file) = cmd.stdout_file {
                                    writeln!(&mut file, "{}", std_out)?;
                                } else if !is_last {
                                    let (stdout_reader, mut stdout_writer) = io::pipe()?;
                                    command_io = Some(stdout_reader);
                                    writeln!(stdout_writer, "{}", std_out)?;
                                } else {
                                    println!("{}", std_out);
                                }
                            }
                        }
                        Err(std_err) => {
                            if let Some(mut file) = cmd.stderr_file {
                                writeln!(&mut file, "{}", std_err)?;
                            } else {
                                println!("{}", std_err);
                            }
                        }
                    }
                } else if find_excutable(&cmd.name).is_none() {
                    println!("{}: command not found", cmd.name);
                } else if cmd.is_background_job {
                    let job_number = self.jobs.new_job_number();
                    let job = cmd.run_as_background_job(command_io, job_number)?;
                    println!("[{}] {}", job.number, job.child.id());
                    self.jobs.push(job);
                    command_io = None;
                } else {
                    match cmd.run_as_excutable(command_io, is_last) {
                        Ok(output) => command_io = output,
                        Err(e) => {
                            println!("{e}");
                            command_io = None;
                        }
                    }
                }
            }
            if !has_job_builtin {
                self.jobs.print_done();
            }
            self.jobs.clean_up();
        }
    }
}
