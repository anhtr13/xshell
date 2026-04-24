use std::{
    env::{home_dir, set_current_dir},
    path::Path,
};

use crate::builtin::BuiltinOutput;

pub fn invoke(args: &[String]) -> BuiltinOutput {
    let mut home = String::new();
    if args.is_empty() || args[0].as_bytes().first() == Some(&b'~') {
        if let Some(h) = home_dir() {
            home = h.display().to_string();
        } else {
            return BuiltinOutput::new(1, "".to_string(), "Impossible to get home dir".to_string());
        }
    }
    let path_string = if args.is_empty() {
        home
    } else if args[0].as_bytes().first() == Some(&b'~') {
        format!("{}{}", home, &args[0][1..].to_string())
    } else {
        args[0].to_string()
    };
    match set_current_dir(Path::new(&path_string)) {
        Ok(_) => BuiltinOutput::default(),
        Err(_) => BuiltinOutput::new(
            1,
            "".to_string(),
            format!("cd: {}: No such file or directory", path_string),
        ),
    }
}
