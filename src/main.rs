mod builtin;
mod command;
mod job;
mod parser;
mod readline;
mod shell;

use crate::{
    job::Jobs,
    readline::{helper::Helper, history::History},
    shell::Shell,
};
use rustyline::{Config, Editor, config::Configurer};
use std::{collections::HashMap, env};

fn main() -> anyhow::Result<()> {
    let mut history = History::default();
    if let Ok(histfile) = env::var("HISTFILE") {
        history.append_from_file(&histfile)?;
        history.set_histfile(histfile);
    };

    let config = Config::builder()
        .bell_style(rustyline::config::BellStyle::Audible)
        .completion_type(rustyline::CompletionType::List)
        .build();

    let jobs = Jobs::new();
    let completers = HashMap::new();
    let variables = HashMap::new();

    let mut editor = Editor::<Helper, History>::with_history(config, history)?;
    let helper = Helper::new(completers);
    editor.set_helper(Some(helper));
    editor.set_auto_add_history(true);

    let mut shell = Shell::new(&mut editor, jobs, variables);
    match shell.run() {
        Ok(_) => {}
        Err(e) => eprintln!("Error: {e}"),
    }
    drop(shell);

    let history = editor.history();
    if let Some(path) = history.histfile() {
        history.write_to_file(path)?;
    }

    Ok(())
}
