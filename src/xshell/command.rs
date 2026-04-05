use std::{
    fs::File,
    io::{self, PipeReader},
    process::{Command, Stdio},
    sync::{Arc, Mutex, mpsc::Sender},
    thread, time,
};

use crate::xshell::{Job, JobStatus};

#[derive(Debug)]
pub struct ShellCommand {
    name: String,
    args: Vec<String>,
    stdout_file: Option<File>,
    stderr_file: Option<File>,
    is_background_job: bool,
}

impl ShellCommand {
    pub fn new(
        name: String,
        args: Vec<String>,
        stdout_file: Option<File>,
        stderr_file: Option<File>,
        is_background_job: bool,
    ) -> Self {
        Self {
            name,
            args,
            stdout_file,
            stderr_file,
            is_background_job,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn args(&self) -> &[String] {
        &self.args
    }
    pub fn stdout_file(&self) -> Option<&File> {
        self.stdout_file.as_ref()
    }
    pub fn stderr_file(&self) -> Option<&File> {
        self.stderr_file.as_ref()
    }
    pub fn is_background_job(&self) -> bool {
        self.is_background_job
    }

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
        stdin: Option<PipeReader>,
        sender: Arc<Sender<u32>>,
        job_number: u32,
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

        let pid = child.id();
        let child = Arc::new(Mutex::new(child));
        let child2 = child.clone();

        thread::spawn(move || {
            child2
                .lock()
                .expect("cannot acquire lock")
                .wait()
                .expect("cannot wait for child process");
            sender.send(job_number).expect("cannot send notify");
        });

        Ok(Job {
            pid,
            number: job_number,
            command: format!("{} {}", self.name, self.args.join(" ")),
            status: JobStatus::Running,
            created_at: time::Instant::now(),
        })
    }
}
