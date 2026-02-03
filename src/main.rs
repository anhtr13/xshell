use std::{
    io::{self, Write},
    process::exit,
};

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
                let args: Vec<&str> = input.split_whitespace().collect();
                let cmd = args[0];

                match cmd {
                    "exit" => {
                        break;
                    }
                    "echo" => {
                        let output = args[1..].join(" ");
                        println!("{output}");
                    }
                    "type" => {
                        if args.len() <= 1 {
                            eprintln!(": not found");
                            continue;
                        }
                        match args[1] {
                            "echo" | "exit" | "type" => {
                                println!("{} is a shell builtin", args[1]);
                            }
                            _ => {
                                println!("{}: not found", args[1]);
                            }
                        }
                    }
                    _ => {
                        eprintln!("{cmd}: command not found");
                    }
                }

                print!("$ ");
                io::stdout().flush().unwrap();

                buffer.clear();
            }
        }
    }
}
