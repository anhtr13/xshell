use std::str::FromStr;

use crate::{
    builtin::{Builtin, BuiltinOutput},
    command::get_command_excutable,
};

pub fn invoke(args: &[String]) -> BuiltinOutput {
    let (status, std_out, std_err) = if Builtin::from_str(&args[0]).is_ok() {
        (0, format!("{} is a shell builtin", args[0]), "".to_string())
    } else if let Ok(path) = get_command_excutable(&args[0]) {
        (0, format!("{} is {path}", args[0]), "".to_string())
    } else {
        (1, "".to_string(), format!("{}: not found", args[0]))
    };
    BuiltinOutput::new(status, std_out, std_err)
}
