use super::{ast, tacky};

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
        let name = format!("var.{}", self.variable_counter);
        self.variable_counter += 1;

        tacky::Variable { identifier: name }
    }

    fn fresh_label(&mut self, suffix: Option<&str>) -> tacky::Label {
        let name = match suffix {
            Some(suffix) => format!("label.{}.{}", self.label_counter, suffix),
            None => format!("label.{}", self.label_counter),
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
            instructions: self.handle_statement(&function.body),
        }
    }

    fn handle_statement(&mut self, statement: &ast::Statement) -> Vec<tacky::Instruction> {
        match statement {
            ast::Statement::Return(expr) => {
                let mut ins = vec![];
                let val = self.handle_expression(&mut ins, expr);
                ins.push(tacky::Instruction::Return(val));

                ins
            }
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
                let op = Self::handle_unary_operator(op.clone());

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
                    let op = Self::handle_binary_operator(op.clone());

                    ins.push(tacky::Instruction::Binary {
                        op,
                        lhs,
                        rhs,
                        dst: dst.clone(),
                    });

                    tacky::Value::Variable(dst)
                }
            },
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
