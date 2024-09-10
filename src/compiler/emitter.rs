use super::asm::{Function, Instruction, Operand, Program, Reg, UnaryOperator};

pub fn emit(program: Program) -> String {
    emit_program(program)
}

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

    format!(
        "\t.globl {prefixed}
{prefixed}:
\tpushq %rbp
\tmovq %rsp, %rbp
{instructions}
"
    )
}

fn emit_instruction(instruction: &Instruction) -> String {
    match instruction {
        Instruction::Mov { src, dst } => {
            format!("\tmovl {}, {}", emit_operand(src), emit_operand(dst))
        }
        Instruction::Ret => "\tmovq %rbp, %rsp
\tpopq %rbp
\tret"
            .to_string(),
        Instruction::Unary { op, dst } => {
            format!("\t{} {}", emit_unary_operator(op), emit_operand(dst))
        }
        Instruction::AllocateStack(size) => format!("\tsubq ${size}, %rsp"),
        _ => todo!(),
    }
}

fn emit_unary_operator(operator: &UnaryOperator) -> String {
    match operator {
        UnaryOperator::Neg => "negl".to_string(),
        UnaryOperator::Not => "notl".to_string(),
    }
}

fn emit_operand(operand: &Operand) -> String {
    match operand {
        Operand::Reg(Reg::AX) => "%eax".to_string(),
        Operand::Reg(Reg::R10) => "%r10d".to_string(),
        Operand::Stack(offset) => format!("-{offset}(%rbp)"),
        Operand::Imm(value) => format!("${}", value),
        Operand::Pseudo(_) => unreachable!(),
        _ => todo!(),
    }
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
                        dst: Operand::Reg(Reg::AX),
                    },
                    Instruction::Ret,
                ],
            },
        };

        let expected = "\t.globl _main
_main:
\tpushq %rbp
\tmovq %rsp, %rbp
\tmovl $42, %eax
\tmovq %rbp, %rsp
\tpopq %rbp
\tret
";

        assert_eq!(emit(program), expected);
    }
}
