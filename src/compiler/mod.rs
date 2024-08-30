use std::path::PathBuf;

pub fn compile(input: &PathBuf, output: &PathBuf) {
    let _str = std::fs::read_to_string(input).unwrap();

    let stub = "\t.globl _main\n_main:\n\tmovl\t$2, %eax\n\tret\n";

    std::fs::write(output, stub).unwrap();
}
