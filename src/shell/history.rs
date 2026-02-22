use std::{
    fs::OpenOptions,
    io::{self, BufRead, BufReader, Write},
};

#[derive(Debug)]
pub struct History {
    pub commands: Vec<String>,
}

impl History {
    pub fn new() -> Self {
        History {
            commands: Vec::new(),
        }
    }

    pub fn add(&mut self, cmd: String) {
        self.commands.push(cmd);
    }

    pub fn append_from_file(&mut self, file_name: &str) -> io::Result<()> {
        let file = OpenOptions::new().read(true).open(file_name)?;
        let reader = BufReader::new(file);
        reader.lines().map_while(Result::ok).for_each(|line| {
            if !line.is_empty() {
                self.commands.push(line);
            }
        });
        Ok(())
    }

    pub fn write_to_file(&self, file_name: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_name)?;
        self.commands.iter().for_each(|line| {
            writeln!(file, "{}", line).expect("Cannot write to file");
        });
        Ok(())
    }

    pub fn append_to_file(&self, file_name: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_name)?;
        let mut idx = 0;
        for i in (0..self.commands.len() - 1).rev() {
            if self.commands[i].starts_with("history -a") {
                idx = i + 1;
                break;
            }
        }
        self.commands.iter().skip(idx).for_each(|cmd| {
            writeln!(file, "{}", cmd).expect("Cannot write to file");
        });
        Ok(())
    }
}
