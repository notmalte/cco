use crate::compiler::ast::Program;

mod identifier_resolution;
mod label_resolution;
mod loop_labeling;

use identifier_resolution::IdentifierResolver;
use label_resolution::LabelResolver;
use loop_labeling::LoopLabeler;

pub fn analyze(program: &Program) -> Result<Program, String> {
    IdentifierResolver::analyze(program)
        .and_then(|program| LabelResolver::analyze(&program))
        .and_then(|program| LoopLabeler::analyze(&program))
}
