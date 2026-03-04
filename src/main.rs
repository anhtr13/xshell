mod shell;

use crate::shell::{helper::InputHelper, history::History};
use rustyline::{Config, Editor, Result, config::Configurer};
use std::env;

fn main() -> Result<()> {
    let mut history = History::default();
    if let Ok(histfile) = env::var("HISTFILE") {
        history.append_from_file(&histfile)?;
        history.history_path = histfile.clone();
    };

    let config = Config::builder()
        .bell_style(rustyline::config::BellStyle::Audible)
        .completion_type(rustyline::CompletionType::List)
        .build();

    let helper = InputHelper::default();

    let mut rl = Editor::<InputHelper, History>::with_history(config, history)?;
    rl.set_helper(Some(helper));
    rl.set_auto_add_history(true);

    match shell::run(&mut rl) {
        Ok(_) => {}
        Err(e) => eprintln!("Error: {e}"),
    }

    let history = rl.history();
    if !history.history_path.is_empty() {
        history.write_to_file(&history.history_path)?;
    }
    Ok(())
}
