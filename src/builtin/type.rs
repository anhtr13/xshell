use std::str::FromStr;

use anyhow::Result;

use crate::{
    builtin::{Builtin, BuiltinOutput},
    command::get_command_excutable,
};

pub fn invoke(args: Vec<String>) -> Result<BuiltinOutput> {
    let ouput = if Builtin::from_str(&args[0]).is_ok() {
        BuiltinOutput::new(0, format!("{} is a shell builtin", args[0]))
    } else if let Ok(ex_path) = get_command_excutable(&args[0]) {
        BuiltinOutput::new(0, format!("{} is {}", args[0], ex_path))
    } else {
        anyhow::bail!("{}: not found", args[0])
    };
    Ok(ouput)
}
