use std::path::PathBuf;

mod codegen;
mod emitter;
mod lexer;
mod parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerStage {
    Lex,
    Parse,
    Codegen,
    Full,
}

pub fn compile(input: &PathBuf, output: &PathBuf, stage: CompilerStage) {
    if std::env::consts::OS != "macos" {
        panic!("Unsupported OS");
    }

    let str = std::fs::read_to_string(input).unwrap();

    let tokens = lexer::tokenize(&str).unwrap();

    if stage == CompilerStage::Lex {
        return;
    }

    let parsed = parser::parse(&tokens).unwrap();

    if stage == CompilerStage::Parse {
        return;
    }

    let ast = codegen::generate(parsed);

    if stage == CompilerStage::Codegen {
        return;
    }

    let emitted = emitter::emit(ast);

    if stage != CompilerStage::Full {
        return;
    }

    std::fs::write(output, emitted).unwrap();
}
