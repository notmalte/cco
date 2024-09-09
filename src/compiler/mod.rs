mod asm;
mod ast;
mod codegen;
mod emitter;
mod lexer;
mod parser;
mod tacky;
mod tackygen;
mod token;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerStage {
    Lex,
    Parse,
    Tacky,
    Codegen,
    Full,
}

pub fn compile(input: &std::path::PathBuf, output: &std::path::PathBuf, stage: CompilerStage) {
    if std::env::consts::OS != "macos" {
        panic!("Unsupported OS");
    }

    let str = std::fs::read_to_string(input).unwrap();

    let tokens = lexer::tokenize(&str).unwrap();
    if stage == CompilerStage::Lex {
        dbg!(tokens);
        return;
    }

    let ast = parser::parse(&tokens).unwrap();
    if stage == CompilerStage::Parse {
        dbg!(&ast);
        return;
    }

    let tacky = tackygen::generate(&ast);
    if stage == CompilerStage::Tacky {
        dbg!(&tacky);
        return;
    }

    let asm = codegen::generate(&ast);
    if stage == CompilerStage::Codegen {
        dbg!(&asm);
        return;
    }

    let emitted = emitter::emit(asm);

    std::fs::write(output, emitted).unwrap();
}
