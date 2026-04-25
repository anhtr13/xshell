use anyhow::Result;

use crate::builtin::BuiltinOutput;

pub fn invoke(args: Vec<String>) -> Result<BuiltinOutput> {
    Ok(BuiltinOutput::new(0, args.join(" ")))
}
