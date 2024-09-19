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
            ast::Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if let Some(else_branch) = else_branch {
                    let else_label = self.fresh_label(Some("if_else"));
                    let end_label = self.fresh_label(Some("if_end"));

                    let condition_value = self.handle_expression(ins, condition);
                    ins.push(tacky::Instruction::JumpIfZero {
                        condition: condition_value,
                        target: else_label.clone(),
                    });
                    self.handle_statement(ins, then_branch);
                    ins.push(tacky::Instruction::Jump {
                        target: end_label.clone(),
                    });
                    ins.push(tacky::Instruction::Label(else_label));
                    self.handle_statement(ins, else_branch);
                    ins.push(tacky::Instruction::Label(end_label));
                } else {
                    let end_label = self.fresh_label(Some("if_end"));

                    let condition_value = self.handle_expression(ins, condition);
                    ins.push(tacky::Instruction::JumpIfZero {
                        condition: condition_value,
                        target: end_label.clone(),
                    });
                    self.handle_statement(ins, then_branch);
                    ins.push(tacky::Instruction::Label(end_label));
                }
            }
            ast::Statement::Goto(_) => todo!(),
            ast::Statement::Labeled(_, _) => todo!(),
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
            ast::Expression::Unary { op, expr: inner } => match op {
                ast::UnaryOperator::PrefixIncrement | ast::UnaryOperator::PrefixDecrement => {
                    let variable = match *inner.clone() {
                        ast::Expression::Variable(ast::Variable { identifier }) => {
                            tacky::Variable { identifier }
                        }
                        _ => unreachable!(),
                    };

                    let op = match op {
                        ast::UnaryOperator::PrefixIncrement => tacky::BinaryOperator::Add,
                        ast::UnaryOperator::PrefixDecrement => tacky::BinaryOperator::Subtract,
                        _ => unreachable!(),
                    };

                    ins.push(tacky::Instruction::Binary {
                        op,
                        lhs: tacky::Value::Variable(variable.clone()),
                        rhs: tacky::Value::Constant(1),
                        dst: variable.clone(),
                    });

                    tacky::Value::Variable(variable)
                }
                ast::UnaryOperator::PostfixIncrement | ast::UnaryOperator::PostfixDecrement => {
                    let variable = match *inner.clone() {
                        ast::Expression::Variable(ast::Variable { identifier }) => {
                            tacky::Variable { identifier }
                        }
                        _ => unreachable!(),
                    };

                    let prev = self.fresh_variable();

                    ins.push(tacky::Instruction::Copy {
                        src: tacky::Value::Variable(variable.clone()),
                        dst: prev.clone(),
                    });

                    let op = match op {
                        ast::UnaryOperator::PostfixIncrement => tacky::BinaryOperator::Add,
                        ast::UnaryOperator::PostfixDecrement => tacky::BinaryOperator::Subtract,
                        _ => unreachable!(),
                    };

                    ins.push(tacky::Instruction::Binary {
                        op,
                        lhs: tacky::Value::Variable(variable.clone()),
                        rhs: tacky::Value::Constant(1),
                        dst: variable.clone(),
                    });

                    tacky::Value::Variable(prev)
                }

                _ => {
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
            },
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
            ast::Expression::Assignment { op, lhs, rhs } => {
                let lhs_variable = match *lhs.clone() {
                    ast::Expression::Variable(ast::Variable { identifier }) => {
                        tacky::Variable { identifier }
                    }
                    _ => unreachable!(),
                };

                let rhs_value = self.handle_expression(ins, rhs);

                match op {
                    ast::AssignmentOperator::Assign => {
                        ins.push(tacky::Instruction::Copy {
                            src: rhs_value,
                            dst: lhs_variable.clone(),
                        });
                    }
                    _ => {
                        ins.push(tacky::Instruction::Binary {
                            op: Self::handle_assignment_operator(*op),
                            lhs: tacky::Value::Variable(lhs_variable.clone()),
                            rhs: rhs_value,
                            dst: lhs_variable.clone(),
                        });
                    }
                }

                tacky::Value::Variable(lhs_variable)
            }
            ast::Expression::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                let dst = self.fresh_variable();

                let label_else = self.fresh_label(Some("cond_else"));
                let label_end = self.fresh_label(Some("cond_end"));

                let condition_value = self.handle_expression(ins, condition);
                ins.push(tacky::Instruction::JumpIfZero {
                    condition: condition_value,
                    target: label_else.clone(),
                });

                let then_value = self.handle_expression(ins, then_expr);
                ins.push(tacky::Instruction::Copy {
                    src: then_value,
                    dst: dst.clone(),
                });
                ins.push(tacky::Instruction::Jump {
                    target: label_end.clone(),
                });

                ins.push(tacky::Instruction::Label(label_else));
                let else_value = self.handle_expression(ins, else_expr);
                ins.push(tacky::Instruction::Copy {
                    src: else_value,
                    dst: dst.clone(),
                });

                ins.push(tacky::Instruction::Label(label_end));

                tacky::Value::Variable(dst)
            }
        }
    }

    fn handle_unary_operator(op: ast::UnaryOperator) -> tacky::UnaryOperator {
        match op {
            ast::UnaryOperator::Negate => tacky::UnaryOperator::Negate,
            ast::UnaryOperator::Complement => tacky::UnaryOperator::Complement,
            ast::UnaryOperator::Not => tacky::UnaryOperator::Not,
            ast::UnaryOperator::PrefixIncrement
            | ast::UnaryOperator::PrefixDecrement
            | ast::UnaryOperator::PostfixIncrement
            | ast::UnaryOperator::PostfixDecrement => unreachable!(),
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

    fn handle_assignment_operator(op: ast::AssignmentOperator) -> tacky::BinaryOperator {
        match op {
            ast::AssignmentOperator::Assign => unreachable!(),
            ast::AssignmentOperator::AddAssign => tacky::BinaryOperator::Add,
            ast::AssignmentOperator::SubtractAssign => tacky::BinaryOperator::Subtract,
            ast::AssignmentOperator::MultiplyAssign => tacky::BinaryOperator::Multiply,
            ast::AssignmentOperator::DivideAssign => tacky::BinaryOperator::Divide,
            ast::AssignmentOperator::RemainderAssign => tacky::BinaryOperator::Remainder,
            ast::AssignmentOperator::BitwiseAndAssign => tacky::BinaryOperator::BitwiseAnd,
            ast::AssignmentOperator::BitwiseOrAssign => tacky::BinaryOperator::BitwiseOr,
            ast::AssignmentOperator::BitwiseXorAssign => tacky::BinaryOperator::BitwiseXor,
            ast::AssignmentOperator::ShiftLeftAssign => tacky::BinaryOperator::ShiftLeft,
            ast::AssignmentOperator::ShiftRightAssign => tacky::BinaryOperator::ShiftRight,
        }
    }
}

pub fn generate(program: &ast::Program) -> tacky::Program {
    (TackyGen::new()).handle_program(program)
}
