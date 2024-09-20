use crate::compiler::ast::Program;

mod labels;
mod variables;

use labels::LabelResolver;
use variables::VariableResolver;

pub fn analyze(program: &Program) -> Result<Program, String> {
    LabelResolver::new().analyze(&VariableResolver::new().analyze(program)?)
}
