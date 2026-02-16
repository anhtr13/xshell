mod helper;
mod shell;

use std::{io::Write, process::exit};

use rustyline::{Config, Editor, Result, history::DefaultHistory};

use crate::{helper::InputHelper, shell::Shell};

fn main() -> Result<()> {
    let config = Config::builder()
        .bell_style(rustyline::config::BellStyle::Audible)
        .build();
    let helper = InputHelper::default();
    let mut rl = Editor::<InputHelper, DefaultHistory>::with_config(config)?;
    rl.set_helper(Some(helper));

    loop {
        match rl.readline("$ ") {
            Err(e) => {
                eprintln!("Error when reading input: {e}");
                exit(1);
            }
            Ok(input) => match Shell::parse_input(&input) {
                Ok(shell) => {
                    let output = shell.run();

                    if !output.std_out.is_empty() {
                        if shell.stdout_redirects.is_empty() && shell.stdout_appends.is_empty() {
                            println!("{}", output.std_out);
                        } else {
                            let std_out = output.std_out;
                            for mut file in shell.stdout_redirects {
                                writeln!(&mut file, "{std_out}")
                                    .unwrap_or_else(|e| eprintln!("{e}"));
                            }
                            for mut file in shell.stdout_appends {
                                writeln!(&mut file, "{std_out}")
                                    .unwrap_or_else(|e| eprintln!("{e}"));
                            }
                        }
                    }

                    if !output.std_err.is_empty() {
                        if shell.stderr_redirects.is_empty() && shell.stderr_appends.is_empty() {
                            eprintln!("{}", output.std_err);
                        } else {
                            let std_err = output.std_err;
                            for mut file in shell.stderr_redirects {
                                writeln!(&mut file, "{std_err}")
                                    .unwrap_or_else(|e| eprintln!("{e}"));
                            }
                            for mut file in shell.stderr_appends {
                                writeln!(&mut file, "{std_err}")
                                    .unwrap_or_else(|e| eprintln!("{e}"));
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{e}");
                }
            },
        }
    }
}
