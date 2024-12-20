mod asm;
mod ast;
mod codegen;
mod constant_conversion;
mod emitter;
mod lexer;
mod parser;
mod prefixes;
mod semantic;
mod symbols;
mod tacky;
mod tackygen;
mod token;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerStage {
    Lex,
    Parse,
    Validate,
    Tacky,
    Codegen,
    Full,
}

pub fn compile(input: &std::path::PathBuf, output: &std::path::PathBuf, stage: CompilerStage) {
    if std::env::consts::OS != "macos" {
        panic!("Unsupported OS");
    }

    let str = std::fs::read_to_string(input).unwrap();

    let tokens = lexer::tokenize(&str).expect("Error during lexing");
    if stage == CompilerStage::Lex {
        dbg!(tokens);
        return;
    }

    let ast_result = parser::parse(&tokens).expect("Error during parsing");
    if stage == CompilerStage::Parse {
        dbg!(&ast_result);
        return;
    }

    let (validated_ast_result, symbols) =
        semantic::analyze(&ast_result).expect("Error during semantic analysis");
    if stage == CompilerStage::Validate {
        dbg!(&validated_ast_result);
        return;
    }

    let tacky_result = tackygen::generate(&validated_ast_result, &symbols);
    if stage == CompilerStage::Tacky {
        dbg!(&tacky_result);
        return;
    }

    let asm_result = codegen::generate(&tacky_result, &symbols);
    if stage == CompilerStage::Codegen {
        dbg!(&asm_result);
        return;
    }

    let emitted = emitter::emit(&asm_result);

    std::fs::write(output, emitted).unwrap();
}
