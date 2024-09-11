use super::asm::{BinaryOperator, Function, Instruction, Operand, Program, Reg, UnaryOperator};

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
        "\t.globl\t{prefixed}
{prefixed}:
\tpushq\t%rbp
\tmovq\t%rsp, %rbp
{instructions}
"
    )
}

fn emit_instruction(instruction: &Instruction) -> String {
    match instruction {
        Instruction::Mov { src, dst } => {
            format!("\tmovl\t{}, {}", emit_operand(src), emit_operand(dst))
        }
        Instruction::Ret => "\tmovq\t%rbp, %rsp
\tpopq\t%rbp
\tret"
            .to_string(),
        Instruction::Unary { op, dst } => {
            format!("\t{}\t{}", emit_unary_operator(op), emit_operand(dst))
        }
        Instruction::Binary { op, src, dst } => {
            format!(
                "\t{}\t{}, {}",
                emit_binary_operator(op),
                emit_operand(src),
                emit_operand(dst)
            )
        }
        Instruction::Idiv(operand) => format!("\tidivl\t{}", emit_operand(operand)),
        Instruction::Cdq => "\tcdq".to_string(),
        Instruction::Sal(operand) => format!("\tsall\t%cl, {}", emit_operand(operand)),
        Instruction::Sar(operand) => format!("\tsarl\t%cl, {}", emit_operand(operand)),
        Instruction::AllocateStack(size) => format!("\tsubq\t${size}, %rsp"),
    }
}

fn emit_unary_operator(operator: &UnaryOperator) -> String {
    match operator {
        UnaryOperator::Neg => "negl".to_string(),
        UnaryOperator::Not => "notl".to_string(),
    }
}

fn emit_binary_operator(operator: &BinaryOperator) -> String {
    match operator {
        BinaryOperator::Add => "addl".to_string(),
        BinaryOperator::Sub => "subl".to_string(),
        BinaryOperator::Mult => "imull".to_string(),
        BinaryOperator::And => "andl".to_string(),
        BinaryOperator::Or => "orl\t".to_string(),
        BinaryOperator::Xor => "xorl".to_string(),
    }
}

fn emit_operand(operand: &Operand) -> String {
    match operand {
        Operand::Reg(Reg::AX) => "%eax".to_string(),
        Operand::Reg(Reg::CX) => "%ecx".to_string(),
        Operand::Reg(Reg::DX) => "%edx".to_string(),
        Operand::Reg(Reg::R10) => "%r10d".to_string(),
        Operand::Reg(Reg::R11) => "%r11d".to_string(),
        Operand::Stack(offset) => format!("-{offset}(%rbp)"),
        Operand::Imm(value) => format!("${}", value),
        Operand::Pseudo(_) => unreachable!(),
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

        let expected = "\t.globl\t_main
_main:
\tpushq\t%rbp
\tmovq\t%rsp, %rbp
\tmovl\t$42, %eax
\tmovq\t%rbp, %rsp
\tpopq\t%rbp
\tret
";

        assert_eq!(emit(program), expected);
    }
}
