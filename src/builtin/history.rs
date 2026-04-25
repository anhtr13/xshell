use anyhow::Result;

use crate::{builtin::BuiltinOutput, readline::history::History};

pub fn invoke(args: Vec<String>, history: &mut History) -> Result<BuiltinOutput> {
    let mut skip = 0;
    if !args.is_empty() {
        if let Ok(limit) = args[0].parse::<usize>() {
            skip = history.commands().len() - limit.min(history.commands().len());
        } else if args.len() >= 2 {
            if args[0] == "-r" {
                history.append_from_file(&args[1])?;
                return Ok(BuiltinOutput::default());
            } else if args[0] == "-w" {
                history.write_to_file(&args[1])?;
                return Ok(BuiltinOutput::default());
            } else if args[0] == "-a" {
                history.append_to_file(&args[1])?;
                return Ok(BuiltinOutput::default());
            }
        }
    }
    let mut stdout = String::new();
    history
        .commands()
        .iter()
        .enumerate()
        .skip(skip)
        .for_each(|(i, cmd)| {
            if i + 1 == history.commands().len() {
                stdout.push_str(&format!("{:>5}  {}", i + 1, cmd));
            } else {
                stdout.push_str(&format!("{:>5}  {}\n", i + 1, cmd));
            }
        });
    Ok(BuiltinOutput::new(0, stdout))
}
