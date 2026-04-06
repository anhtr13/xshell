mod xshell;

use crate::xshell::{Shell, helper::InputHelper, history::History};
use rustyline::{Config, Editor, config::Configurer};
use std::env;

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

    let helper = InputHelper::default();

    let mut editor = Editor::<InputHelper, History>::with_history(config, history)?;
    editor.set_helper(Some(helper));
    editor.set_auto_add_history(true);

    let mut shell = Shell::new(&mut editor);
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
