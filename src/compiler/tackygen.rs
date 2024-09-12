use super::{
    ast,
    constants::{TAC_LABEL_PREFIX, TAC_VAR_PREFIX},
    tacky,
};

pub struct TackyGen {
    variable_counter: usize,
    label_counter: usize,
}

impl TackyGen {
    fn new() -> Self {
        Self {
            variable_counter: 0,
            label_counter: 0,
        }
    }

    fn fresh_variable(&mut self) -> tacky::Variable {
        let name = format!("{TAC_VAR_PREFIX}.{}", self.variable_counter);
        self.variable_counter += 1;

        tacky::Variable { identifier: name }
    }

    fn fresh_label(&mut self, suffix: Option<&str>) -> tacky::Label {
        let name = match suffix {
            Some(suffix) => format!("{TAC_LABEL_PREFIX}.{}.{}", self.label_counter, suffix),
            None => format!("{TAC_LABEL_PREFIX}.{}", self.label_counter),
        };
        self.label_counter += 1;

        tacky::Label { identifier: name }
    }

    fn handle_program(&mut self, program: &ast::Program) -> tacky::Program {
        tacky::Program {
            function_definition: self.handle_function(&program.function_definition),
        }
    }

    fn handle_function(&mut self, function: &ast::Function) -> tacky::Function {
        tacky::Function {
            name: function.name.clone(),
            instructions: self.handle_block_items(&function.body),
        }
    }

    fn handle_block_items(&mut self, body: &[ast::BlockItem]) -> Vec<tacky::Instruction> {
        let mut ins = vec![];

        for item in body {
            match item {
                ast::BlockItem::Declaration(declaration) => {
                    self.handle_declaration(&mut ins, declaration);
                }
                ast::BlockItem::Statement(statement) => {
                    self.handle_statement(&mut ins, statement);
                }
            }
        }

        ins.push(tacky::Instruction::Return(tacky::Value::Constant(0)));

        ins
    }

    fn handle_declaration(
        &mut self,
        ins: &mut Vec<tacky::Instruction>,
        declaration: &ast::Declaration,
    ) {
        if let Some(initializer) = &declaration.initializer {
            let value = self.handle_expression(ins, initializer);
            let variable = tacky::Variable {
                identifier: declaration.variable.identifier.clone(),
            };
            ins.push(tacky::Instruction::Copy {
                src: value,
                dst: variable,
            });
        }
    }

    fn handle_statement(&mut self, ins: &mut Vec<tacky::Instruction>, statement: &ast::Statement) {
        match statement {
            ast::Statement::Return(expr) => {
                let value = self.handle_expression(ins, expr);
                ins.push(tacky::Instruction::Return(value));
            }
            ast::Statement::Expression(expr) => {
                self.handle_expression(ins, expr);
            }
            ast::Statement::Null => {}
        }
    }

    fn handle_expression(
        &mut self,
        ins: &mut Vec<tacky::Instruction>,
        expr: &ast::Expression,
    ) -> tacky::Value {
        match expr {
            ast::Expression::Constant(value) => tacky::Value::Constant(*value),
            ast::Expression::Unary { op, expr: inner } => {
                let src = self.handle_expression(ins, inner);
                let dst = self.fresh_variable();
                let op = Self::handle_unary_operator(*op);

                ins.push(tacky::Instruction::Unary {
                    op,
                    src,
                    dst: dst.clone(),
                });

                tacky::Value::Variable(dst)
            }
            ast::Expression::Binary { op, lhs, rhs } => match op {
                ast::BinaryOperator::LogicalAnd => {
                    let dst = self.fresh_variable();

                    let label_false = self.fresh_label(Some("and_false"));
                    let label_end = self.fresh_label(Some("and_end"));

                    let lhs = self.handle_expression(ins, lhs);
                    ins.push(tacky::Instruction::JumpIfZero {
                        condition: lhs,
                        target: label_false.clone(),
                    });

                    let rhs = self.handle_expression(ins, rhs);
                    ins.push(tacky::Instruction::JumpIfZero {
                        condition: rhs,
                        target: label_false.clone(),
                    });

                    ins.push(tacky::Instruction::Copy {
                        src: tacky::Value::Constant(1),
                        dst: dst.clone(),
                    });
                    ins.push(tacky::Instruction::Jump {
                        target: label_end.clone(),
                    });

                    ins.push(tacky::Instruction::Label(label_false));

                    ins.push(tacky::Instruction::Copy {
                        src: tacky::Value::Constant(0),
                        dst: dst.clone(),
                    });

                    ins.push(tacky::Instruction::Label(label_end));

                    tacky::Value::Variable(dst)
                }
                ast::BinaryOperator::LogicalOr => {
                    let dst = self.fresh_variable();

                    let label_true = self.fresh_label(Some("or_true"));
                    let label_end = self.fresh_label(Some("or_end"));

                    let lhs = self.handle_expression(ins, lhs);
                    ins.push(tacky::Instruction::JumpIfNotZero {
                        condition: lhs,
                        target: label_true.clone(),
                    });

                    let rhs = self.handle_expression(ins, rhs);
                    ins.push(tacky::Instruction::JumpIfNotZero {
                        condition: rhs,
                        target: label_true.clone(),
                    });

                    ins.push(tacky::Instruction::Copy {
                        src: tacky::Value::Constant(0),
                        dst: dst.clone(),
                    });
                    ins.push(tacky::Instruction::Jump {
                        target: label_end.clone(),
                    });

                    ins.push(tacky::Instruction::Label(label_true));

                    ins.push(tacky::Instruction::Copy {
                        src: tacky::Value::Constant(1),
                        dst: dst.clone(),
                    });

                    ins.push(tacky::Instruction::Label(label_end));

                    tacky::Value::Variable(dst)
                }
                _ => {
                    let lhs = self.handle_expression(ins, lhs);
                    let rhs = self.handle_expression(ins, rhs);
                    let dst = self.fresh_variable();
                    let op = Self::handle_binary_operator(*op);

                    ins.push(tacky::Instruction::Binary {
                        op,
                        lhs,
                        rhs,
                        dst: dst.clone(),
                    });

                    tacky::Value::Variable(dst)
                }
            },
            ast::Expression::Variable(ast::Variable { identifier }) => {
                tacky::Value::Variable(tacky::Variable {
                    identifier: identifier.clone(),
                })
            }
            ast::Expression::Assignment { lhs, rhs } => {
                let value = self.handle_expression(ins, rhs);
                let variable = match *lhs.clone() {
                    ast::Expression::Variable(ast::Variable { identifier }) => {
                        tacky::Variable { identifier }
                    }
                    _ => unreachable!(),
                };

                ins.push(tacky::Instruction::Copy {
                    src: value,
                    dst: variable.clone(),
                });

                tacky::Value::Variable(variable)
            }
        }
    }

    fn handle_unary_operator(op: ast::UnaryOperator) -> tacky::UnaryOperator {
        match op {
            ast::UnaryOperator::Negate => tacky::UnaryOperator::Negate,
            ast::UnaryOperator::Complement => tacky::UnaryOperator::Complement,
            ast::UnaryOperator::Not => tacky::UnaryOperator::Not,
        }
    }

    fn handle_binary_operator(op: ast::BinaryOperator) -> tacky::BinaryOperator {
        match op {
            ast::BinaryOperator::Add => tacky::BinaryOperator::Add,
            ast::BinaryOperator::Subtract => tacky::BinaryOperator::Subtract,
            ast::BinaryOperator::Multiply => tacky::BinaryOperator::Multiply,
            ast::BinaryOperator::Divide => tacky::BinaryOperator::Divide,
            ast::BinaryOperator::Remainder => tacky::BinaryOperator::Remainder,
            ast::BinaryOperator::BitwiseAnd => tacky::BinaryOperator::BitwiseAnd,
            ast::BinaryOperator::BitwiseOr => tacky::BinaryOperator::BitwiseOr,
            ast::BinaryOperator::BitwiseXor => tacky::BinaryOperator::BitwiseXor,
            ast::BinaryOperator::ShiftLeft => tacky::BinaryOperator::ShiftLeft,
            ast::BinaryOperator::ShiftRight => tacky::BinaryOperator::ShiftRight,
            ast::BinaryOperator::Equal => tacky::BinaryOperator::Equal,
            ast::BinaryOperator::NotEqual => tacky::BinaryOperator::NotEqual,
            ast::BinaryOperator::LessThan => tacky::BinaryOperator::LessThan,
            ast::BinaryOperator::LessOrEqual => tacky::BinaryOperator::LessOrEqual,
            ast::BinaryOperator::GreaterThan => tacky::BinaryOperator::GreaterThan,
            ast::BinaryOperator::GreaterOrEqual => tacky::BinaryOperator::GreaterOrEqual,
            ast::BinaryOperator::LogicalAnd | ast::BinaryOperator::LogicalOr => unreachable!(),
        }
    }
}

pub fn generate(program: &ast::Program) -> tacky::Program {
    (TackyGen::new()).handle_program(program)
}
