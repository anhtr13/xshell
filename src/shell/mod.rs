mod parser;

use std::{
    collections::HashMap,
    io::{self, Write},
    str::FromStr,
    sync::{
        Arc, RwLock,
        mpsc::{self, Sender},
    },
    thread,
};

use rustyline::Editor;

use crate::{
    builtin::{self, Builtin},
    command::get_command_excutable,
    job::{Job, JobStatus, recent_jobs_ids},
    readline::{helper::InputHelper, history::History},
    shell::parser::parse_commands_from_input,
};

pub struct Shell<'a> {
    editor: &'a mut Editor<InputHelper, History>,
    jobs: Arc<RwLock<HashMap<u32, Job>>>,
    tx_done: Arc<Sender<u32>>, // channel for notifying done jobs
}

impl<'a> Shell<'a> {
    pub fn new(editor: &'a mut Editor<InputHelper, History>) -> Self {
        let jobs = Arc::new(RwLock::new(HashMap::<u32, Job>::new()));
        let jobs2 = jobs.clone();
        let (tx, rx) = mpsc::channel::<u32>();
        thread::spawn(move || {
            loop {
                let id = rx.recv();
                match id {
                    Ok(id) => {
                        let mut jobs = jobs2.write().unwrap();
                        if let Some(job) = jobs.get_mut(&id) {
                            job.status = JobStatus::Done
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            editor,
            jobs,
            tx_done: Arc::new(tx),
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            let input = self.editor.readline("$ ")?;
            let commands = parse_commands_from_input(input)?;
            let total_commands = commands.len();
            let mut command_io = None;

            for (idx, cmd) in commands.into_iter().enumerate() {
                let is_last = idx + 1 == total_commands;
                if let Ok(builtin) = Builtin::from_str(cmd.name()) {
                    let output = match builtin {
                        Builtin::Cd => builtin::run_cd(cmd.args()),
                        Builtin::Echo => builtin::run_echo(cmd.args()),
                        Builtin::History => {
                            builtin::run_history(cmd.args(), self.editor.history_mut())
                        }
                        Builtin::Pwd => builtin::run_pwd(),
                        Builtin::Type => builtin::run_type(cmd.args()),
                        Builtin::Jobs => builtin::run_job(self.get_all_jobs()),
                        Builtin::Complete => return Ok(()),
                        Builtin::Exit => return Ok(()),
                    };
                    if !output.std_err().is_empty() {
                        if let Some(mut file) = cmd.stderr_file() {
                            writeln!(&mut file, "{}", output.std_err())?;
                        } else {
                            println!("{}", output.std_err());
                        }
                    }
                    command_io = None;
                    if !output.std_out().is_empty() {
                        if let Some(mut file) = cmd.stdout_file() {
                            writeln!(&mut file, "{}", output.std_out())?;
                        } else if !is_last {
                            let (stdout_reader, mut stdout_writer) = io::pipe()?;
                            command_io = Some(stdout_reader);
                            writeln!(stdout_writer, "{}", output.std_out())?;
                        } else {
                            println!("{}", output.std_out());
                        }
                    }
                    if builtin == Builtin::Jobs {
                        self.clean_jobs();
                        break;
                    }
                } else if let Err(e) = get_command_excutable(cmd.name()) {
                    eprintln!("{e}");
                    break;
                } else if cmd.is_background_job() {
                    let job = cmd.run_as_background_job(
                        command_io,
                        self.tx_done.clone(),
                        self.new_job_number(),
                    )?;
                    println!("[{}] {}", job.number, job.pid);
                    self.add_job(job);
                    command_io = None;
                } else {
                    command_io = cmd.run_as_external_command(command_io, is_last)?;
                }
            }
            self.print_done_jobs();
            self.clean_jobs();
        }
    }

    fn new_job_number(&self) -> u32 {
        let read_jobs = self.jobs.read().unwrap();
        let mut num = 1;
        while read_jobs.contains_key(&num) {
            num += 1;
        }
        num
    }

    fn add_job(&mut self, job: Job) {
        let mut write_jobs = self.jobs.write().unwrap();
        write_jobs.insert(job.number, job);
    }

    fn get_all_jobs(&self) -> Vec<Job> {
        let read_jobs = self.jobs.read().unwrap();
        let jobs: Vec<_> = read_jobs.values().cloned().collect();
        jobs
    }

    fn print_done_jobs(&self) {
        let mut jobs = self.get_all_jobs();
        if jobs.is_empty() {
            return;
        }
        let recent = recent_jobs_ids(&jobs);
        jobs.sort_unstable_by_key(|x| x.number);
        for job in jobs.iter().filter(|job| job.status == JobStatus::Done) {
            let marker = match job.pid {
                id if id == recent.0 => "+",
                id if id == recent.1 => "-",
                _ => " ",
            };
            println!(
                "[{}]{}  Done                    {}",
                job.number, marker, job.command
            );
        }
    }

    fn clean_jobs(&mut self) {
        let mut write_jobs = self.jobs.write().unwrap();
        write_jobs.retain(|_, job| job.status == JobStatus::Running);
    }
}
