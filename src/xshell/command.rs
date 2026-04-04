use std::{
    fs::File,
    io::{self, PipeReader},
    process::{Child, Command, Stdio},
};

#[derive(Debug)]
pub struct ShellCommand {
    pub name: String,
    pub args: Vec<String>,
    pub stdout_file: Option<File>,
    pub stderr_file: Option<File>,
    pub is_background_job: bool,
}

impl ShellCommand {
    pub fn run_external(
        self,
        stdin: Option<PipeReader>,
        is_last: bool,
    ) -> anyhow::Result<(Child, Option<PipeReader>)> {
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

        Ok((child, output))
    }
}
