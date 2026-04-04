pub mod builtin;
pub mod command;
pub mod helper;
pub mod history;
mod utils;

use std::{
    collections::HashMap,
    fmt::Display,
    io::{self, Write},
    str::FromStr,
    sync::{
        Arc, Mutex,
        mpsc::{self, Sender},
    },
    thread,
};

use rustyline::Editor;

use crate::xshell::{
    builtin::Builtin,
    helper::InputHelper,
    history::History,
    utils::{check_command_excutable, commands_from_input},
};

#[derive(PartialEq)]
enum JobStatus {
    Running,
    Done,
}

impl Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "Running"),
            Self::Done => write!(f, "Done"),
        }
    }
}

#[allow(unused)]
pub struct Job {
    id: u32,
    process: u32,
    command: String,
    status: JobStatus,
}

pub struct Shell<'a> {
    editor: &'a mut Editor<InputHelper, History>,
    jobs: Arc<Mutex<HashMap<u32, Job>>>,
    sender: Arc<Sender<u32>>,
}

impl<'a> Shell<'a> {
    pub fn new(editor: &'a mut Editor<InputHelper, History>) -> Self {
        let jobs = Arc::new(Mutex::new(HashMap::<u32, Job>::new()));
        let jobs2 = jobs.clone();
        let (sender, receiver) = mpsc::channel::<u32>();
        thread::spawn(move || {
            loop {
                let id = receiver.recv();
                match id {
                    Ok(id) => {
                        let mut jobs = jobs2.lock().unwrap();
                        if let Some(job) = jobs.get_mut(&id) {
                            job.status = JobStatus::Done
                        }
                        // if let Some(job) = jobs.remove(&id) {
                        //     print!("\r\x1b[2K");
                        //     io::stdout().flush().unwrap();
                        //     println!("[{}]+  Done                    {}", job.job_id, job.command);
                        //     print!("$ ");
                        //     io::stdout().flush().unwrap();
                        // }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        });
        Self {
            editor,
            jobs,
            sender: Arc::new(sender),
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            let input = self.editor.readline("$ ")?;
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
                        Builtin::History => {
                            builtin::run_history(&cmd.args, self.editor.history_mut())
                        }
                        Builtin::Pwd => builtin::run_pwd(),
                        Builtin::Type => builtin::run_type(&cmd.args),
                        Builtin::Jobs => builtin::run_job(self.jobs.clone()),
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
                } else if cmd.is_background_job {
                    let mut jobs = self.jobs.lock().unwrap();
                    let job_id = jobs.len() as u32 + 1;
                    let job = cmd.run_as_background_job(job_id, command_io, self.sender.clone())?;
                    println!("[{}] {}", job_id, job.process);
                    jobs.insert(job_id, job);
                    command_io = None;
                } else {
                    command_io = cmd.run_as_external_command(command_io, is_last)?;
                }
            }
        }
    }
}
