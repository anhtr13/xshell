mod helper;
mod shell;

use std::{io::Write, str::FromStr};

use rustyline::{Config, Editor, Result, history::DefaultHistory};

use crate::{
    helper::InputHelper,
    shell::{Builtin, parse_input},
};

fn main() -> Result<()> {
    let config = Config::builder()
        .bell_style(rustyline::config::BellStyle::Audible)
        .completion_type(rustyline::CompletionType::List)
        .build();
    let helper = InputHelper::default();
    let mut rl = Editor::<InputHelper, DefaultHistory>::with_config(config)?;
    rl.set_helper(Some(helper));

    loop {
        let input = rl.readline("$ ")?;
        match parse_input(&input) {
            Ok(cmds) => {
                let mut prev_child_stdout = None;
                let count_cmds = cmds.len();
                for (idx, cmd) in cmds.into_iter().enumerate() {
                    if let Ok(builtin) = Builtin::from_str(&cmd.name) {
                        let output = builtin.run(&cmd.args);
                        if !output.std_out.is_empty() {
                            if idx + 1 == count_cmds
                                && cmd.stdout_overwrite.is_empty()
                                && cmd.stdout_appends.is_empty()
                            {
                                println!("{}", output.std_out);
                            } else {
                                cmd.stdout_overwrite.into_iter().for_each(|mut file| {
                                    writeln!(&mut file, "{}", output.std_out)
                                        .unwrap_or_else(|e| eprintln!("{e}"));
                                });
                                cmd.stdout_appends.into_iter().for_each(|mut file| {
                                    writeln!(&mut file, "{}", output.std_out)
                                        .unwrap_or_else(|e| eprintln!("{e}"));
                                });
                            }
                        }
                        if !output.std_err.is_empty() {
                            if idx + 1 == count_cmds
                                && cmd.stderr_overwrite.is_empty()
                                && cmd.stderr_appends.is_empty()
                            {
                                println!("{}", output.std_err);
                            } else {
                                cmd.stderr_overwrite.into_iter().for_each(|mut file| {
                                    writeln!(&mut file, "{}", output.std_err)
                                        .unwrap_or_else(|e| eprintln!("{e}"));
                                });
                                cmd.stderr_appends.into_iter().for_each(|mut file| {
                                    writeln!(&mut file, "{}", output.std_err)
                                        .unwrap_or_else(|e| eprintln!("{e}"));
                                });
                            }
                        }
                    } else {
                        match cmd.run(prev_child_stdout, idx + 1 == count_cmds) {
                            Ok(mut child) => {
                                // let ouput = child.wait_with_output().unwrap();
                                // let stdout = String::from_utf8(ouput.stdout).unwrap_or_default();
                                // let stderr = String::from_utf8(ouput.stderr).unwrap_or_default();
                                // if idx + 1 == count_cmds
                                //     && cmd.stdout_overwrite.is_empty()
                                //     && cmd.stdout_appends.is_empty()
                                // {
                                //     print!("{stdout}");
                                // } else {
                                //     cmd.stdout_overwrite.into_iter().for_each(|mut file| {
                                //         writeln!(&mut file, "{}", stdout)
                                //             .unwrap_or_else(|e| eprintln!("{e}"));
                                //     });
                                //     cmd.stdout_appends.into_iter().for_each(|mut file| {
                                //         writeln!(&mut file, "{}", stdout)
                                //             .unwrap_or_else(|e| eprintln!("{e}"));
                                //     });
                                // }
                                // if idx + 1 == count_cmds
                                //     && cmd.stderr_overwrite.is_empty()
                                //     && cmd.stderr_appends.is_empty()
                                // {
                                //     print!("{stderr}");
                                // } else {
                                //     cmd.stderr_overwrite.into_iter().for_each(|mut file| {
                                //         writeln!(&mut file, "{}", stderr)
                                //             .unwrap_or_else(|e| eprintln!("{e}"));
                                //     });
                                //     cmd.stderr_appends.into_iter().for_each(|mut file| {
                                //         writeln!(&mut file, "{}", stderr)
                                //             .unwrap_or_else(|e| eprintln!("{e}"));
                                //     });
                                // }

                                if idx + 1 == count_cmds {
                                    let _ = child.wait()?;
                                    prev_child_stdout = None;
                                } else {
                                    prev_child_stdout = child.stdout;
                                }
                            }
                            Err(e) => {
                                prev_child_stdout = None;
                                eprintln!("Error: {e}");
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        }
    }
}
