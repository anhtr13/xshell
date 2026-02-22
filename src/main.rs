mod shell;

use crate::shell::{
    builtin::Builtin, check_is_excutable, helper::InputHelper, history::History, parse_input,
};
use rustyline::{
    Config, Editor, Result,
    config::Configurer,
    history::{DefaultHistory, FileHistory},
};
use std::{env, str::FromStr};

fn run(mut rl: Editor<InputHelper, FileHistory>, mut history: History) -> Result<()> {
    loop {
        let input = rl.readline("$ ")?;
        match parse_input(&input) {
            Ok(cmds) => {
                let mut cmd_io = None;
                let total_cmds = cmds.len();

                for (idx, cmd) in cmds.into_iter().enumerate() {
                    rl.add_history_entry(&input)?;
                    history.add(format!("{} {}", cmd.name, cmd.args.join(" ")));

                    let is_last = idx + 1 == total_cmds;

                    if let Ok(builtin) = Builtin::from_str(&cmd.name) {
                        cmd_io = builtin.run(cmd, &mut history, is_last);
                    } else {
                        if let Err(e) = check_is_excutable(&cmd.name) {
                            eprintln!("{e}");
                            break;
                        }
                        cmd_io = cmd.run(cmd_io, is_last);
                    }
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        }
    }
}

fn main() -> Result<()> {
    let config = Config::builder()
        .bell_style(rustyline::config::BellStyle::Audible)
        .completion_type(rustyline::CompletionType::List)
        .build();
    let helper = InputHelper::default();
    let mut rl = Editor::<InputHelper, DefaultHistory>::with_config(config)?;
    rl.set_helper(Some(helper));
    rl.set_auto_add_history(true);

    let mut history = History::new();
    if let Ok(histfile) = env::var("HISTFILE") {
        history.append_from_file(&histfile)?;
    };

    run(rl, history)
}
