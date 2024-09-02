use std::path::PathBuf;

mod lexer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerStage {
    Lex,
    Parse,
    Codegen,
    Full,
}

pub fn compile(input: &PathBuf, output: &PathBuf, stage: CompilerStage) {
    let str = std::fs::read_to_string(input).unwrap();

    let tokens = lexer::tokenize(&str);

    dbg!(tokens);

    let stub = "\t.globl _main\n_main:\n\tmovl\t$2, %eax\n\tret\n";

    if stage != CompilerStage::Full {
        return;
    }

    std::fs::write(output, stub).unwrap();
}
