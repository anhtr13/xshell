use std::collections::HashMap;

use anyhow::Result;

use crate::builtin::BuiltinOutput;

pub fn invoke(
    mut args: Vec<String>,
    completers: &mut HashMap<String, String>,
) -> Result<BuiltinOutput> {
    if args.len() >= 2 {
        match args[0].as_str() {
            "-C" => {
                anyhow::ensure!(args.len() == 3);
                let command = args.pop().unwrap();
                let completer = args.pop().unwrap();
                completers.insert(command, completer);
            }
            "-p" => {
                anyhow::ensure!(args.len() == 2);
                if let Some(complete_path) = completers.get(&args[1]) {
                    return Ok(BuiltinOutput::new(
                        0,
                        format!("complete -C '{}' {}", complete_path, args[1]),
                    ));
                } else {
                    return Ok(BuiltinOutput::new(
                        0,
                        format!("complete: {}: no completion specification", args[1]),
                    ));
                }
            }
            "-r" => {
                anyhow::ensure!(args.len() == 2);
                completers.remove(&args[1]);
            }
            _ => {}
        }
    }
    Ok(BuiltinOutput::default())
}
