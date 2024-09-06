use super::parser::{Expression, Program as AstProgram, Statement};

#[derive(Debug)]
pub struct Program {
    function_definition: Function,
}

#[derive(Debug)]
pub struct Function {
    name: String,
    instructions: Vec<Instruction>,
}

#[derive(Debug)]
pub enum Instruction {
    Mov { src: Operand, dst: Operand },
    Ret,
}

#[derive(Debug)]
pub enum Operand {
    Imm(i32),
    Register,
}

pub fn generate(ast_program: AstProgram) -> Program {
    return Program {
        function_definition: Function {
            name: ast_program.function_definition.name,
            instructions: match ast_program.function_definition.body {
                Statement::Return(expr) => match expr {
                    Expression::IntLiteral(value) => vec![
                        Instruction::Mov {
                            src: Operand::Imm(value),
                            dst: Operand::Register,
                        },
                        Instruction::Ret,
                    ],
                },
            },
        },
    };
}
