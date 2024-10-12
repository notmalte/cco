use crate::compiler::asm::{
    BinaryOperator, ConditionCode, FunctionDefinition, Instruction, Label, Operand, Program, Reg,
    TopLevelItem, UnaryOperator,
};

use super::asm::StaticVariable;

pub fn emit(program: &Program) -> String {
    emit_program(program)
}

fn emit_program(program: &Program) -> String {
    program
        .items
        .iter()
        .map(emit_top_level_item)
        .collect::<Vec<_>>()
        .join("\n")
}

fn emit_top_level_item(item: &TopLevelItem) -> String {
    match item {
        TopLevelItem::FunctionDefinition(fd) => emit_function_definition(fd),
        TopLevelItem::StaticVariable(sv) => emit_static_variable(sv),
    }
}

fn prefix_identifier(identifier: &str) -> String {
    format!("_{identifier}",)
}

fn build_global_directive(identifier: &str, global: bool) -> String {
    if global {
        format!("\t.globl\t{identifier}\n")
    } else {
        "".to_string()
    }
}

fn emit_function_definition(fd: &FunctionDefinition) -> String {
    let prefixed = prefix_identifier(&fd.function.identifier);

    let instructions = fd
        .instructions
        .iter()
        .map(emit_instruction)
        .collect::<Vec<_>>()
        .join("\n");

    let global_directive = build_global_directive(&prefixed, fd.global);

    format!(
        "{global_directive}\t.text
{prefixed}:
\tpushq\t%rbp
\tmovq\t%rsp, %rbp
{instructions}
"
    )
}

fn emit_static_variable(sv: &StaticVariable) -> String {
    let identifier = prefix_identifier(&sv.variable.identifier);
    let initial = sv.initial;
    let global_directive = build_global_directive(&sv.variable.identifier, sv.global);
    let alignment_directive = "\t.balign 4\n";

    if initial == 0 {
        format!(
            "{global_directive}\t.bss
{alignment_directive}{identifier}:
\t.zero 4
"
        )
    } else {
        format!(
            "{global_directive}\t.data
{alignment_directive}{identifier}:
\t.long {initial}
"
        )
    }
}

fn emit_instruction(instruction: &Instruction) -> String {
    match instruction {
        Instruction::Mov { src, dst } => {
            format!(
                "\tmovl\t{}, {}",
                emit_operand(src, RegSize::FourBytes),
                emit_operand(dst, RegSize::FourBytes)
            )
        }
        Instruction::Unary { op, dst } => {
            format!(
                "\t{}\t{}",
                emit_unary_operator(op),
                emit_operand(dst, RegSize::FourBytes)
            )
        }
        Instruction::Binary { op, src, dst } => {
            format!(
                "\t{}\t{}, {}",
                emit_binary_operator(op),
                emit_operand(src, RegSize::FourBytes),
                emit_operand(dst, RegSize::FourBytes)
            )
        }
        Instruction::Cmp { src, dst } => {
            format!(
                "\tcmpl\t{}, {}",
                emit_operand(src, RegSize::FourBytes),
                emit_operand(dst, RegSize::FourBytes)
            )
        }
        Instruction::Idiv(operand) => {
            format!("\tidivl\t{}", emit_operand(operand, RegSize::FourBytes))
        }
        Instruction::Cdq => "\tcdq".to_string(),
        Instruction::Sal(operand) => {
            format!("\tsall\t%cl, {}", emit_operand(operand, RegSize::FourBytes))
        }
        Instruction::Sar(operand) => {
            format!("\tsarl\t%cl, {}", emit_operand(operand, RegSize::FourBytes))
        }
        Instruction::Jmp { target } => format!("\tjmp\t\t{}", emit_label(target)),
        Instruction::JmpCC { cc, target } => {
            format!("\tj{}\t{}", emit_condition_code(cc), emit_label(target))
        }
        Instruction::SetCC { cc, dst } => {
            format!(
                "\tset{}\t{}",
                emit_condition_code(cc),
                emit_operand(dst, RegSize::OneByte)
            )
        }
        Instruction::Label(label) => format!("{}:", emit_label(label)),
        Instruction::AllocateStack(bytes) => format!("\tsubq\t${bytes}, %rsp"),
        Instruction::DeallocateStack(bytes) => format!("\taddq\t${bytes}, %rsp"),
        Instruction::Push(operand) => {
            format!("\tpushq\t{}", emit_operand(operand, RegSize::EightBytes))
        }
        Instruction::Call(function) => {
            format!("\tcall\t{}", prefix_identifier(&function.identifier))
        }
        Instruction::Ret => "\tmovq\t%rbp, %rsp
\tpopq\t%rbp
\tret"
            .to_string(),
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum RegSize {
    OneByte,
    FourBytes,
    EightBytes,
}

fn emit_operand(operand: &Operand, size: RegSize) -> String {
    match operand {
        Operand::Reg(reg) => match size {
            RegSize::OneByte => match reg {
                Reg::AX => "%al",
                Reg::CX => "%cl",
                Reg::DX => "%dl",
                Reg::DI => "%dil",
                Reg::SI => "%sil",
                Reg::R8 => "%r8b",
                Reg::R9 => "%r9b",
                Reg::R10 => "%r10b",
                Reg::R11 => "%r11b",
            },
            RegSize::FourBytes => match reg {
                Reg::AX => "%eax",
                Reg::CX => "%ecx",
                Reg::DX => "%edx",
                Reg::DI => "%edi",
                Reg::SI => "%esi",
                Reg::R8 => "%r8d",
                Reg::R9 => "%r9d",
                Reg::R10 => "%r10d",
                Reg::R11 => "%r11d",
            },
            RegSize::EightBytes => match reg {
                Reg::AX => "%rax",
                Reg::CX => "%rcx",
                Reg::DX => "%rdx",
                Reg::DI => "%rdi",
                Reg::SI => "%rsi",
                Reg::R8 => "%r8",
                Reg::R9 => "%r9",
                Reg::R10 => "%r10",
                Reg::R11 => "%r11",
            },
        }
        .to_string(),
        Operand::Stack(offset) => format!("{offset}(%rbp)"),
        Operand::Imm(value) => format!("${}", value),
        Operand::Data(identifier) => format!("{}(%rip)", prefix_identifier(identifier)),
        Operand::Pseudo(_) => unreachable!(),
    }
}

fn emit_label(label: &Label) -> String {
    format!("L{}", label.identifier)
}

fn emit_condition_code(cc: &ConditionCode) -> String {
    match cc {
        ConditionCode::E => "e".to_string(),
        ConditionCode::NE => "ne".to_string(),
        ConditionCode::L => "l".to_string(),
        ConditionCode::LE => "le".to_string(),
        ConditionCode::G => "g".to_string(),
        ConditionCode::GE => "ge".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::compiler::asm::Function;

    #[test]
    fn test_emit() {
        let program = Program {
            items: vec![TopLevelItem::FunctionDefinition(FunctionDefinition {
                function: Function {
                    identifier: "main".to_string(),
                },
                global: true,
                instructions: vec![
                    Instruction::Mov {
                        src: Operand::Imm(42),
                        dst: Operand::Reg(Reg::AX),
                    },
                    Instruction::Ret,
                ],
            })],
        };

        let expected = "\t.globl\t_main
\t.text
_main:
\tpushq\t%rbp
\tmovq\t%rsp, %rbp
\tmovl\t$42, %eax
\tmovq\t%rbp, %rsp
\tpopq\t%rbp
\tret
";

        assert_eq!(emit(&program), expected);
    }
}
