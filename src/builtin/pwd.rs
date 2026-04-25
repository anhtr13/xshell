use std::env::current_dir;

use anyhow::Result;

use crate::builtin::BuiltinOutput;

pub fn invoke() -> Result<BuiltinOutput> {
    let dir = current_dir()?;
    Ok(BuiltinOutput::new(0, dir.display().to_string()))
}
