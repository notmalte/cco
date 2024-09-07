use super::parser::{
    Expression as AstExpression, Program as AstProgram, Statement as AstStatement,
};

#[derive(Debug, PartialEq)]
pub struct Program {
    function_definition: Function,
}

#[derive(Debug, PartialEq)]
pub struct Function {
    name: String,
    instructions: Vec<Instruction>,
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Mov { src: Operand, dst: Operand },
    Ret,
}

#[derive(Debug, PartialEq)]
pub enum Operand {
    Imm(i32),
    Register,
}

pub fn generate(ast_program: AstProgram) -> Program {
    return Program {
        function_definition: Function {
            name: ast_program.function_definition.name,
            instructions: match ast_program.function_definition.body {
                AstStatement::Return(expr) => match expr {
                    AstExpression::IntLiteral(value) => vec![
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

#[cfg(test)]
mod tests {
    use super::{super::parser::Function as AstFunction, *};

    #[test]
    fn test_generate() {
        let ast_program = AstProgram {
            function_definition: AstFunction {
                name: "main".to_string(),
                body: AstStatement::Return(AstExpression::IntLiteral(42)),
            },
        };

        let program = generate(ast_program);

        assert_eq!(
            program,
            Program {
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
            }
        );
    }
}
