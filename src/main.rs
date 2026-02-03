use std::io::{self, Write};

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut cmd = String::new();
    io::stdin().read_line(&mut cmd).unwrap();
    if cmd != "" {
        eprintln!("{cmd}: command not found");
        return;
    }
}
