use std::collections::HashMap;

use super::{asm, tacky};

pub fn generate(program: &tacky::Program) -> asm::Program {
    let mut p = handle_program(program);

    let stack_size = replace_pseudo_registers(&mut p);

    fix_up_instructions(&mut p, stack_size);

    p
}

fn handle_program(program: &tacky::Program) -> asm::Program {
    asm::Program {
        function_definition: handle_function(&program.function_definition),
    }
}

fn handle_function(function: &tacky::Function) -> asm::Function {
    asm::Function {
        name: function.name.clone(),
        instructions: handle_instructions(&function.instructions),
    }
}

fn handle_instructions(instructions: &[tacky::Instruction]) -> Vec<asm::Instruction> {
    let mut ins = vec![];

    for instruction in instructions {
        match instruction {
            tacky::Instruction::Return(value) => {
                ins.push(asm::Instruction::Mov {
                    src: handle_value(value),
                    dst: asm::Operand::Reg(asm::Reg::AX),
                });
                ins.push(asm::Instruction::Ret);
            }
            tacky::Instruction::Unary { op, src, dst } => {
                let dst_asm = handle_variable(dst);
                ins.push(asm::Instruction::Mov {
                    src: handle_value(src),
                    dst: dst_asm.clone(),
                });
                ins.push(asm::Instruction::Unary {
                    op: handle_unary_operator(op),
                    dst: dst_asm,
                });
            }
            tacky::Instruction::Binary { op, lhs, rhs, dst } => match op {
                tacky::BinaryOperator::Add
                | tacky::BinaryOperator::Subtract
                | tacky::BinaryOperator::Multiply
                | tacky::BinaryOperator::BitwiseAnd
                | tacky::BinaryOperator::BitwiseOr
                | tacky::BinaryOperator::BitwiseXor => {
                    let dst_asm = handle_variable(dst);
                    ins.push(asm::Instruction::Mov {
                        src: handle_value(lhs),
                        dst: dst_asm.clone(),
                    });
                    ins.push(asm::Instruction::Binary {
                        op: handle_binary_operator(op),
                        src: handle_value(rhs),
                        dst: dst_asm,
                    });
                }
                tacky::BinaryOperator::Divide => {
                    ins.push(asm::Instruction::Mov {
                        src: handle_value(lhs),
                        dst: asm::Operand::Reg(asm::Reg::AX),
                    });
                    ins.push(asm::Instruction::Cdq);
                    ins.push(asm::Instruction::Idiv(handle_value(rhs)));
                    ins.push(asm::Instruction::Mov {
                        src: asm::Operand::Reg(asm::Reg::AX),
                        dst: handle_variable(dst),
                    });
                }
                tacky::BinaryOperator::Remainder => {
                    ins.push(asm::Instruction::Mov {
                        src: handle_value(lhs),
                        dst: asm::Operand::Reg(asm::Reg::AX),
                    });
                    ins.push(asm::Instruction::Cdq);
                    ins.push(asm::Instruction::Idiv(handle_value(rhs)));
                    ins.push(asm::Instruction::Mov {
                        src: asm::Operand::Reg(asm::Reg::DX),
                        dst: handle_variable(dst),
                    });
                }
                tacky::BinaryOperator::ShiftLeft | tacky::BinaryOperator::ShiftRight => {
                    let dst_asm = handle_variable(dst);
                    ins.push(asm::Instruction::Mov {
                        src: handle_value(lhs),
                        dst: dst_asm.clone(),
                    });
                    ins.push(asm::Instruction::Mov {
                        src: handle_value(rhs),
                        dst: asm::Operand::Reg(asm::Reg::CX).clone(),
                    });
                    ins.push(match op {
                        tacky::BinaryOperator::ShiftLeft => asm::Instruction::Sal(dst_asm),
                        tacky::BinaryOperator::ShiftRight => asm::Instruction::Sar(dst_asm),
                        _ => unreachable!(),
                    });
                }
            },
        }
    }

    ins
}

fn handle_value(value: &tacky::Value) -> asm::Operand {
    match value {
        tacky::Value::Constant(value) => asm::Operand::Imm(*value),
        tacky::Value::Variable(variable) => handle_variable(variable),
    }
}

fn handle_variable(variable: &tacky::Variable) -> asm::Operand {
    asm::Operand::Pseudo(variable.identifier.clone())
}

fn handle_unary_operator(op: &tacky::UnaryOperator) -> asm::UnaryOperator {
    match op {
        tacky::UnaryOperator::Complement => asm::UnaryOperator::Not,
        tacky::UnaryOperator::Negate => asm::UnaryOperator::Neg,
    }
}

fn handle_binary_operator(op: &tacky::BinaryOperator) -> asm::BinaryOperator {
    match op {
        tacky::BinaryOperator::Add => asm::BinaryOperator::Add,
        tacky::BinaryOperator::Subtract => asm::BinaryOperator::Sub,
        tacky::BinaryOperator::Multiply => asm::BinaryOperator::Mult,
        tacky::BinaryOperator::BitwiseAnd => asm::BinaryOperator::And,
        tacky::BinaryOperator::BitwiseOr => asm::BinaryOperator::Or,
        tacky::BinaryOperator::BitwiseXor => asm::BinaryOperator::Xor,
        _ => unreachable!(),
    }
}

fn replace_pseudo_registers(program: &mut asm::Program) -> u64 {
    let mut map = HashMap::<String, u64>::new();

    for ins in &mut program.function_definition.instructions {
        match ins {
            asm::Instruction::Mov { src, dst } => {
                replace_pseudo_registers_in_operand(src, &mut map);
                replace_pseudo_registers_in_operand(dst, &mut map);
            }
            asm::Instruction::Unary { op: _, dst } => {
                replace_pseudo_registers_in_operand(dst, &mut map);
            }
            asm::Instruction::Binary { op: _, src, dst } => {
                replace_pseudo_registers_in_operand(src, &mut map);
                replace_pseudo_registers_in_operand(dst, &mut map);
            }
            asm::Instruction::Idiv(operand) => {
                replace_pseudo_registers_in_operand(operand, &mut map);
            }
            asm::Instruction::Sal(operand) | asm::Instruction::Sar(operand) => {
                replace_pseudo_registers_in_operand(operand, &mut map);
            }
            asm::Instruction::Ret | asm::Instruction::Cdq => {}
            asm::Instruction::AllocateStack(_) => unreachable!(),
        }
    }

    4 * (map.len() as u64)
}

fn replace_pseudo_registers_in_operand(operand: &mut asm::Operand, map: &mut HashMap<String, u64>) {
    if let asm::Operand::Pseudo(name) = operand {
        let candidate = 4 * ((map.len() as u64) + 1);
        let offset = *map.entry(name.clone()).or_insert(candidate);
        *operand = asm::Operand::Stack(offset);
    }
}

fn fix_up_instructions(program: &mut asm::Program, stack_size: u64) {
    let mut ins = vec![];

    ins.push(asm::Instruction::AllocateStack(stack_size));

    for i in &program.function_definition.instructions {
        match i {
            asm::Instruction::Mov {
                src: src @ asm::Operand::Stack(_),
                dst: dst @ asm::Operand::Stack(_),
            } => {
                ins.push(asm::Instruction::Mov {
                    src: src.clone(),
                    dst: asm::Operand::Reg(asm::Reg::R10),
                });
                ins.push(asm::Instruction::Mov {
                    src: asm::Operand::Reg(asm::Reg::R10),
                    dst: dst.clone(),
                });
            }
            asm::Instruction::Idiv(value @ asm::Operand::Imm(_)) => {
                ins.push(asm::Instruction::Mov {
                    src: value.clone(),
                    dst: asm::Operand::Reg(asm::Reg::R10),
                });
                ins.push(asm::Instruction::Idiv(asm::Operand::Reg(asm::Reg::R10)));
            }
            asm::Instruction::Binary {
                op:
                    op @ (asm::BinaryOperator::Add
                    | asm::BinaryOperator::Sub
                    | asm::BinaryOperator::And
                    | asm::BinaryOperator::Or
                    | asm::BinaryOperator::Xor),
                src: src @ asm::Operand::Stack(_),
                dst: dst @ asm::Operand::Stack(_),
            } => {
                ins.push(asm::Instruction::Mov {
                    src: src.clone(),
                    dst: asm::Operand::Reg(asm::Reg::R10),
                });
                ins.push(asm::Instruction::Binary {
                    op: op.clone(),
                    src: asm::Operand::Reg(asm::Reg::R10),
                    dst: dst.clone(),
                });
            }
            asm::Instruction::Binary {
                op: asm::BinaryOperator::Mult,
                src,
                dst: dst @ asm::Operand::Stack(_),
            } => {
                ins.push(asm::Instruction::Mov {
                    src: dst.clone(),
                    dst: asm::Operand::Reg(asm::Reg::R11),
                });
                ins.push(asm::Instruction::Binary {
                    op: asm::BinaryOperator::Mult,
                    src: src.clone(),
                    dst: asm::Operand::Reg(asm::Reg::R11),
                });
                ins.push(asm::Instruction::Mov {
                    src: asm::Operand::Reg(asm::Reg::R11),
                    dst: dst.clone(),
                });
            }
            i => ins.push(i.clone()),
        }
    }

    program.function_definition.instructions = ins;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let tacky_program = tacky::Program {
            function_definition: tacky::Function {
                name: "main".to_string(),
                instructions: vec![tacky::Instruction::Return(tacky::Value::Constant(42))],
            },
        };

        let program = generate(&tacky_program);

        assert_eq!(
            program,
            asm::Program {
                function_definition: asm::Function {
                    name: "main".to_string(),
                    instructions: vec![
                        asm::Instruction::AllocateStack(0),
                        asm::Instruction::Mov {
                            src: asm::Operand::Imm(42),
                            dst: asm::Operand::Reg(asm::Reg::AX),
                        },
                        asm::Instruction::Ret,
                    ],
                },
            }
        );
    }
}
