use std::{
    io::{self, Write},
    path::Path,
    process::exit,
};

fn type_cmd(args: &[&str]) {
    if args.is_empty() {
        eprintln!("not found");
        return;
    }
    let cmd_name = args[0];
    match cmd_name {
        "echo" | "exit" | "type" => {
            println!("{} is a shell builtin", args[1]);
        }
        _ => {
            let path = std::env::var("PATH").expect("cannot get PATH");
            let bins: Vec<&str> = path.split(':').collect();
            for bin in bins {
                let p = format!("{bin}/{cmd_name}");
                let path = Path::new(&p);
                if path.is_file() {
                    println!("{cmd_name} is {bin}/{cmd_name}");
                    return;
                }
            }
            println!("{}: not found", args[1]);
        }
    }
}

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
                        type_cmd(&args[1..]);
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
