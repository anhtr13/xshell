use crate::builtin::BuiltinOutput;

pub fn invoke(args: &[String]) -> BuiltinOutput {
    BuiltinOutput::new(0, args.join(" "), "".to_string())
}
