use std::{
    borrow::Cow,
    fs::OpenOptions,
    io::{self, BufRead, BufReader, Write},
};

use rustyline::history::History as Rustyline_History;

#[derive(Debug, Default)]
pub struct History {
    pub commands: Vec<String>,
    pub history_path: String,
    max_length: usize,
    ignore_dup: bool,
    ignore_space: bool,
}

impl History {
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
        for line in self.commands.iter() {
            writeln!(file, "{}", line)?;
        }
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
        for cmd in self.commands.iter().skip(idx) {
            writeln!(file, "{}", cmd)?;
        }
        Ok(())
    }
}

impl Rustyline_History for History {
    fn get(
        &self,
        index: usize,
        _dir: rustyline::history::SearchDirection,
    ) -> rustyline::Result<Option<rustyline::history::SearchResult<'_>>> {
        let n = self.commands.len();
        if index >= n {
            return Ok(None);
        }
        let cmd = self.commands[index].clone();
        Ok(Some(rustyline::history::SearchResult {
            entry: Cow::Owned(cmd),
            idx: index,
            pos: 0,
        }))
    }
    fn add(&mut self, line: &str) -> rustyline::Result<bool> {
        let line = line.trim();
        if line.is_empty() {
            return Ok(false);
        }
        if self.ignore_space
            && let Some(c) = line.chars().nth(0)
            && c == ' '
        {
            return Ok(false);
        }
        if self.ignore_dup
            && let Some(last) = self.commands.last()
            && last == line
        {
            return Ok(false);
        }
        self.commands.push(line.to_string());
        Ok(true)
    }
    fn add_owned(&mut self, line: String) -> rustyline::Result<bool> {
        let line = line.trim();
        if line.is_empty() {
            return Ok(false);
        }
        if self.ignore_space
            && let Some(c) = line.chars().nth(0)
            && c == ' '
        {
            return Ok(false);
        }
        if self.ignore_dup
            && let Some(last) = self.commands.last()
            && last == line
        {
            return Ok(false);
        }
        self.commands.push(line.to_string());
        Ok(true)
    }
    fn len(&self) -> usize {
        self.commands.len()
    }
    fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
    fn set_max_len(&mut self, len: usize) -> rustyline::Result<()> {
        self.max_length = len;
        Ok(())
    }
    fn ignore_dups(&mut self, yes: bool) -> rustyline::Result<()> {
        self.ignore_dup = yes;
        Ok(())
    }
    fn ignore_space(&mut self, yes: bool) {
        self.ignore_space = yes
    }
    fn save(&mut self, path: &std::path::Path) -> rustyline::Result<()> {
        self.write_to_file(&path.display().to_string())?;
        Ok(())
    }
    fn append(&mut self, path: &std::path::Path) -> rustyline::Result<()> {
        self.append_from_file(&path.display().to_string())?;
        Ok(())
    }
    fn load(&mut self, path: &std::path::Path) -> rustyline::Result<()> {
        self.append_from_file(&path.display().to_string())?;
        Ok(())
    }
    fn clear(&mut self) -> rustyline::Result<()> {
        while self.commands.is_empty() {
            self.commands.pop();
        }
        Ok(())
    }
    fn search(
        &self,
        term: &str,
        start: usize,
        dir: rustyline::history::SearchDirection,
    ) -> rustyline::Result<Option<rustyline::history::SearchResult<'_>>> {
        let n = self.commands.len();
        match dir {
            rustyline::history::SearchDirection::Forward => {
                for idx in (start..n).rev() {
                    let line = &self.commands[idx];
                    if let Some(pos) = line.find(term) {
                        return Ok(Some(rustyline::history::SearchResult {
                            entry: Cow::Owned(line.clone()),
                            idx,
                            pos,
                        }));
                    }
                }
            }
            rustyline::history::SearchDirection::Reverse => {
                for idx in 0..=start {
                    let line = &self.commands[idx];
                    if let Some(pos) = line.find(term) {
                        return Ok(Some(rustyline::history::SearchResult {
                            entry: Cow::Owned(line.clone()),
                            idx,
                            pos,
                        }));
                    }
                }
            }
        }
        Ok(None)
    }
    fn starts_with(
        &self,
        term: &str,
        start: usize,
        dir: rustyline::history::SearchDirection,
    ) -> rustyline::Result<Option<rustyline::history::SearchResult<'_>>> {
        let n = self.commands.len();
        match dir {
            rustyline::history::SearchDirection::Forward => {
                for idx in (start..n).rev() {
                    let line = &self.commands[idx];
                    if line.starts_with(term) {
                        return Ok(Some(rustyline::history::SearchResult {
                            entry: Cow::Owned(line.clone()),
                            idx,
                            pos: term.len(),
                        }));
                    }
                }
            }
            rustyline::history::SearchDirection::Reverse => {
                for idx in 0..=start {
                    let line = &self.commands[idx];
                    if line.starts_with(term) {
                        return Ok(Some(rustyline::history::SearchResult {
                            entry: Cow::Owned(line.clone()),
                            idx,
                            pos: term.len(),
                        }));
                    }
                }
            }
        }
        Ok(None)
    }
}
