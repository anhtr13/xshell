use std::{
    fs::metadata,
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, exit},
};

const BUILTIN: &[&str] = &["exit", "echo", "type"];

fn find_excutable(name: &str) -> Option<String> {
    let path = std::env::var("PATH").expect("cannot get PATH");
    let bins: Vec<&str> = path.split(':').collect();
    for bin in bins {
        let p = format!("{bin}/{name}");
        let path = Path::new(&p);
        if path.is_file() {
            let mode = metadata(path).unwrap().permissions().mode();
            if mode & 0o100 != 0 || mode & 0o010 != 0 || mode & 0o001 != 0 {
                return Some(format!("{bin}/{name}"));
            }
        }
    }
    None
}

fn run_executable(path: &str, args: &[&str]) -> String {
    let output = Command::new(path)
        .args(args)
        .output()
        .expect("Failed to execute command");
    let stdout = str::from_utf8(&output.stdout).expect("Invalid UTF-8");
    return stdout.to_string();
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
                        if BUILTIN.contains(&args[1]) {
                            println!("{} is a shell builtin", args[1]);
                        } else if let Some(path) = find_excutable(args[1]) {
                            println!("{} is {path}", args[1])
                        } else {
                            println!("{}: not found", args[1]);
                        }
                    }
                    _ => {
                        if let Some(_) = find_excutable(cmd) {
                            let stdout = run_executable(args[0], &args[1..]);
                            print!("{stdout}");
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
