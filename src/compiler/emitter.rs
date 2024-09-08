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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit() {
        let program = Program {
            function_definition: Function {
                name: "main".to_string(),
                instructions: vec![
                    Instruction::Mov {
                        src: Operand::Imm(42),
                        dst: Operand::Register,
                    },
                    Instruction::Ret,
                ],
            },
        };

        let expected = "\t.globl _main\n_main:\n\tmovl $42, %eax\n\tret\n";

        assert_eq!(emit(program), expected);
    }
}
