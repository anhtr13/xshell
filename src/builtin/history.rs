use crate::{builtin::BuiltinOutput, readline::history::History};

pub fn invoke(args: &[String], history: &mut History) -> BuiltinOutput {
    let mut skip = 0;
    if !args.is_empty() {
        if let Ok(limit) = args[0].parse::<usize>() {
            skip = history.commands().len() - limit.min(history.commands().len());
        } else if args.len() >= 2 {
            if args[0] == "-r" {
                match history.append_from_file(&args[1]) {
                    Ok(_) => {
                        return BuiltinOutput::default();
                    }
                    Err(e) => {
                        return BuiltinOutput::new(1, "".to_string(), e.to_string());
                    }
                }
            } else if args[0] == "-w" {
                match history.write_to_file(&args[1]) {
                    Ok(_) => {
                        return BuiltinOutput::default();
                    }
                    Err(e) => {
                        return BuiltinOutput::new(1, "".to_string(), e.to_string());
                    }
                }
            } else if args[0] == "-a" {
                match history.append_to_file(&args[1]) {
                    Ok(_) => {
                        return BuiltinOutput::default();
                    }
                    Err(e) => {
                        return BuiltinOutput::new(1, "".to_string(), e.to_string());
                    }
                }
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
    BuiltinOutput {
        status: 0,
        std_out: stdout,
        std_err: "".to_string(),
    }
}
