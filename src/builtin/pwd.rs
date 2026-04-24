use std::env::current_dir;

use crate::builtin::BuiltinOutput;

pub fn invoke() -> BuiltinOutput {
    match current_dir() {
        Ok(path) => BuiltinOutput::new(0, path.display().to_string(), "".to_string()),
        Err(e) => BuiltinOutput::new(1, "".to_string(), e.to_string()),
    }
}
