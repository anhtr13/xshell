use std::{
    fs::File,
    io::{self, PipeReader},
    process::{Command, Stdio},
};

#[derive(Debug)]
pub struct Cmd {
    pub name: String,
    pub args: Vec<String>,
    pub stdout_file: Option<File>,
    pub stderr_file: Option<File>,
}

impl Cmd {
    pub fn run(
        self,
        stdin: Option<PipeReader>,
        history: &mut Vec<String>,
        is_last: bool,
    ) -> Option<PipeReader> {
        let stdin = if let Some(stdio) = stdin {
            Stdio::from(stdio)
        } else {
            Stdio::inherit()
        };

        let mut output = None;

        let stdout = if let Some(stdout_file) = self.stdout_file {
            Stdio::from(stdout_file)
        } else if !is_last {
            let (stdout_reader, stdout_writer) = io::pipe().expect("Cannot create pipe");
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
            .spawn()
            .expect("Cannot spawn process");

        if is_last {
            let _ = child.wait();
        }

        history.push(format!("{} {}", self.name, self.args.join(" ")));

        output
    }
}
