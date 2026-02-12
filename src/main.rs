mod shell;
mod utils;

use std::{
    io::{self, Write},
    process::exit,
    str::FromStr,
};

use crate::shell::{Builtin, Output};

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut buffer = String::new();

    loop {
        match io::stdin().read_line(&mut buffer) {
            Err(e) => {
                eprintln!("Error when reading input: {e}");
                exit(1);
            }
            Ok(_) => {
                let input = buffer.trim();
                match utils::parse_input(input) {
                    Ok(cli) => {
                        let output = if let Ok(cmd) = shell::Builtin::from_str(&cli.cmd) {
                            match cmd {
                                Builtin::Cd => shell::run_cd(&cli.args),
                                Builtin::Echo => shell::run_echo(&cli.args),
                                Builtin::Exit => break,
                                Builtin::Pwd => shell::run_pwd(),
                                Builtin::Type => shell::run_type(&cli.args),
                            }
                        } else if utils::find_excutable(&cli.cmd).is_some() {
                            utils::run_executable(&cli.cmd, &cli.args)
                        } else {
                            eprintln!("{}: command not found", cli.cmd);
                            Output {
                                status: 0,
                                std_out: "".to_string(),
                                std_err: "".to_string(),
                            }
                        };

                        if !output.std_out.is_empty() {
                            if cli.stdout_redirects.is_empty() && cli.stdout_appends.is_empty() {
                                println!("{}", output.std_out);
                            } else {
                                let std_out = output.std_out;
                                for mut file in cli.stdout_redirects {
                                    file.write_all(std_out.as_bytes())
                                        .unwrap_or_else(|e| eprintln!("{e}"));
                                }
                                for mut file in cli.stdout_appends {
                                    writeln!(&mut file, "{std_out}")
                                        .unwrap_or_else(|e| eprintln!("{e}"));
                                }
                            }
                        }

                        if !output.std_err.is_empty() {
                            if cli.stderr_redirects.is_empty() && cli.stderr_appends.is_empty() {
                                eprintln!("{}", output.std_err);
                            } else {
                                let std_err = output.std_err;
                                for mut file in cli.stderr_redirects {
                                    file.write_all(std_err.as_bytes())
                                        .unwrap_or_else(|e| eprintln!("{e}"));
                                }
                                for mut file in cli.stderr_appends {
                                    writeln!(&mut file, "{std_err}")
                                        .unwrap_or_else(|e| eprintln!("{e}"));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{e}");
                    }
                }

                buffer.clear();

                print!("$ ");
                io::stdout().flush().unwrap();
            }
        }
    }
}
