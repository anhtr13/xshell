use std::{
    fs::File,
    io::{self, PipeReader},
    process::{Command, Stdio},
    sync::{Arc, Mutex, mpsc::Sender},
    thread,
};

use crate::xshell::{Job, JobStatus};

#[derive(Debug)]
pub struct ShellCommand {
    pub name: String,
    pub args: Vec<String>,
    pub stdout_file: Option<File>,
    pub stderr_file: Option<File>,
    pub is_background_job: bool,
}

impl ShellCommand {
    pub fn run_as_external_command(
        self,
        stdin: Option<PipeReader>,
        is_last: bool,
    ) -> anyhow::Result<Option<PipeReader>> {
        let stdin = if let Some(stdio) = stdin {
            Stdio::from(stdio)
        } else {
            Stdio::inherit()
        };

        let mut output = None;

        let stdout = if let Some(stdout_file) = self.stdout_file {
            Stdio::from(stdout_file)
        } else if !is_last {
            let (stdout_reader, stdout_writer) = io::pipe()?;
            output = Some(stdout_reader);
            Stdio::from(stdout_writer)
        } else {
            Stdio::inherit()
        };

        let stderr = if let Some(stderr_file) = self.stderr_file {
            Stdio::from(stderr_file)
        } else {
            Stdio::inherit()
        };

        let mut child = Command::new(&self.name)
            .args(&self.args)
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()?;

        if is_last && !self.is_background_job {
            child.wait()?;
        }

        Ok(output)
    }

    pub fn run_as_background_job(
        self,
        job_id: u32,
        stdin: Option<PipeReader>,
        sender: Arc<Sender<u32>>,
    ) -> anyhow::Result<Job> {
        let stdin = if let Some(stdio) = stdin {
            Stdio::from(stdio)
        } else {
            Stdio::inherit()
        };

        let stdout = if let Some(stdout_file) = self.stdout_file {
            Stdio::from(stdout_file)
        } else {
            Stdio::inherit()
        };

        let stderr = if let Some(stderr_file) = self.stderr_file {
            Stdio::from(stderr_file)
        } else {
            Stdio::inherit()
        };

        let child = Command::new(&self.name)
            .args(&self.args)
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()?;

        let process_id = child.id();
        let child = Arc::new(Mutex::new(child));
        let child2 = child.clone();

        thread::spawn(move || {
            child2
                .lock()
                .expect("cannot acquire lock")
                .wait()
                .expect("cannot wait for child process");
            sender.send(job_id).expect("cannot send notify");
        });

        Ok(Job {
            id: job_id,
            process: process_id,
            command: format!("{} {}", self.name, self.args.join(" ")),
            status: JobStatus::Running,
        })
    }
}
