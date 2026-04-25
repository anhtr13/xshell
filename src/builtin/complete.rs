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
                let key = args.pop().unwrap();
                let pattern = args.pop().unwrap();
                completers.insert(key, pattern);
            }
            "-p" => {
                anyhow::ensure!(args.len() == 2);
                if let Some(pattern) = completers.get(&args[1]) {
                    return Ok(BuiltinOutput::new(
                        0,
                        format!("complete -C '{}' {}", pattern, args[1]),
                    ));
                } else {
                    return Ok(BuiltinOutput::new(
                        0,
                        format!("complete: {}: no completion specification", args[1]),
                    ));
                }
            }
            _ => {}
        }
    }
    Ok(BuiltinOutput::default())
}
