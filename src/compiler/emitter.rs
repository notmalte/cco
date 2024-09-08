use super::asm::{Function, Instruction, Operand, Program};

fn emit_program(program: Program) -> String {
    emit_function(program.function_definition)
}

fn emit_function(function: Function) -> String {
    let prefixed = format!("_{}", function.name);

    let instructions = function
        .instructions
        .iter()
        .map(emit_instruction)
        .collect::<Vec<_>>()
        .join("\n");

    format!("\t.globl {}\n{}:\n{}\n", prefixed, prefixed, instructions)
}

fn emit_instruction(instruction: &Instruction) -> String {
    match instruction {
        Instruction::Mov { src, dst } => {
            format!("\tmovl {}, {}", emit_operand(src), emit_operand(dst))
        }
        Instruction::Ret => "\tret".to_string(),
    }
}

fn emit_operand(operand: &Operand) -> String {
    match operand {
        Operand::Imm(value) => format!("${}", value),
        Operand::Register => "%eax".to_string(),
    }
}

pub fn emit(program: Program) -> String {
    emit_program(program)
}
