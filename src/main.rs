use std::{
    fs::File,
    io::{self, Write},
    process::exit,
    str::FromStr,
};

use crate::builtin::Builtin;

mod builtin;
mod utils;

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
                let (cmd, mut args) = utils::parse_input(input).unwrap();
                let mut file = None;

                if let Some(r_idx) = args.iter().position(|x| x == ">" || x == "1>")
                    && r_idx + 1 < args.len()
                    && let Ok(f) = File::create(&args[r_idx + 1])
                {
                    args = args[..r_idx].to_vec();
                    file = Some(f);
                }

                let output = if let Ok(cmd) = Builtin::from_str(&cmd) {
                    match cmd {
                        Builtin::Cd => builtin::run_cd(&args),
                        Builtin::Echo => builtin::run_echo(&args),
                        Builtin::Exit => break,
                        Builtin::Pwd => builtin::run_pwd(),
                        Builtin::Type => builtin::run_type(&args),
                    }
                } else if utils::find_excutable(&cmd).is_some() {
                    utils::run_executable(&cmd, &args)
                } else {
                    Some(format!("{}: command not found", cmd))
                };

                if let Some(std_out) = output {
                    if let Some(mut file) = file {
                        match file.write_all(std_out.as_bytes()) {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("{e}");
                            }
                        }
                    } else {
                        println!("{std_out}");
                    }
                }

                buffer.clear();

                print!("$ ");
                io::stdout().flush().unwrap();
            }
        }
    }
}
