use std::{
    fs::metadata,
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::exit,
};

fn type_cmd(args: &[&str]) {
    if args.is_empty() {
        eprintln!("not found");
        return;
    }
    let cmd_to_check = args[0];
    match cmd_to_check {
        "echo" | "exit" | "type" => {
            println!("{} is a shell builtin", cmd_to_check);
        }
        _ => {
            let path = std::env::var("PATH").expect("cannot get PATH");
            let bins: Vec<&str> = path.split(':').collect();
            for bin in bins {
                let p = format!("{bin}/{cmd_to_check}");
                let path = Path::new(&p);
                if path.is_file() {
                    let mode = metadata(path).unwrap().permissions().mode();
                    if mode & 0o100 != 0 || mode & 0o010 != 0 || mode & 0o001 != 0 {
                        println!("{cmd_to_check} is {bin}/{cmd_to_check}");
                        return;
                    }
                }
            }
            println!("{}: not found", cmd_to_check);
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
