use super::{asm, ast};

fn generate_program(program: ast::Program) -> asm::Program {
    asm::Program {
        function_definition: generate_function(program.function_definition),
    }
}

fn generate_function(function: ast::Function) -> asm::Function {
    asm::Function {
        name: function.name,
        instructions: generate_instructions_from_statement(function.body),
    }
}

fn generate_instructions_from_statement(statement: ast::Statement) -> Vec<asm::Instruction> {
    match statement {
        ast::Statement::Return(expr) => match expr {
            ast::Expression::IntLiteral(value) => vec![
                asm::Instruction::Mov {
                    src: asm::Operand::Imm(value),
                    dst: asm::Operand::Register,
                },
                asm::Instruction::Ret,
            ],
        },
    }
}

pub fn generate(program: ast::Program) -> asm::Program {
    generate_program(program)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let ast_program = ast::Program {
            function_definition: ast::Function {
                name: "main".to_string(),
                body: ast::Statement::Return(ast::Expression::IntLiteral(42)),
            },
        };

        let program = generate(ast_program);

        assert_eq!(
            program,
            asm::Program {
                function_definition: asm::Function {
                    name: "main".to_string(),
                    instructions: vec![
                        asm::Instruction::Mov {
                            src: asm::Operand::Imm(42),
                            dst: asm::Operand::Register,
                        },
                        asm::Instruction::Ret,
                    ],
                },
            }
        );
    }
}
