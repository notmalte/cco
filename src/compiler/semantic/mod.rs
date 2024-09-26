use crate::compiler::ast::Program;

mod labels;
mod loops;
mod variables;

use labels::LabelResolver;
use loops::LoopLabeler;
use variables::VariableResolver;

pub fn analyze(program: &Program) -> Result<Program, String> {
    VariableResolver::analyze(program)
        .and_then(|program| LabelResolver::analyze(&program))
        .and_then(|program| LoopLabeler::analyze(&program))
}
