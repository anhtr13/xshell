use std::{
    io::{self, Write},
    process::exit,
};

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
                let (cmd, args) = utils::parse_input(input).unwrap();

                match cmd {
                    "exit" => {
                        break;
                    }
                    "echo" => {
                        builtin::run_echo(&args);
                    }
                    "type" => {
                        builtin::run_type(&args);
                    }
                    "pwd" => {
                        builtin::run_pwd();
                    }
                    "cd" => {
                        builtin::run_cd(&args);
                    }
                    _ => {
                        if utils::find_excutable(&cmd).is_some() {
                            let _ = utils::run_executable(cmd, &args);
                        } else {
                            eprintln!("{}: command not found", cmd);
                        }
                    }
                }

                buffer.clear();

                print!("$ ");
                io::stdout().flush().unwrap();
            }
        }
    }
}
