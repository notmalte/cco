use crate::compiler::{ast::Program, symbols::SymbolTable};

mod identifier_resolution;
mod label_resolution;
mod loop_switch_labeling;
mod type_check;

use identifier_resolution::IdentifierResolver;
use label_resolution::LabelResolver;
use loop_switch_labeling::LoopSwitchLabeler;
use type_check::TypeChecker;

pub fn analyze(program: &Program) -> Result<(Program, SymbolTable), String> {
    IdentifierResolver::analyze(program)
        .and_then(|program| LabelResolver::analyze(&program))
        .and_then(|program| LoopSwitchLabeler::analyze(&program))
        .and_then(|program| TypeChecker::analyze(&program))
}
