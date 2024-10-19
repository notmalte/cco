use crate::compiler::{
    ast,
    constants::{TAC_LABEL_PREFIX, TAC_VAR_PREFIX},
    symbols::{SymbolAttributes, SymbolInitialValue, SymbolTable},
    tacky,
};

pub fn generate(program: &ast::Program, symbols: &SymbolTable) -> tacky::Program {
    (TackyGen::new()).handle_program(program, symbols)
}

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

    fn break_label(label: &ast::LoopOrSwitchLabel) -> tacky::Label {
        tacky::Label {
            identifier: format!(
                "{}.break",
                match label {
                    ast::LoopOrSwitchLabel::Loop(loop_label) => &loop_label.identifier,
                    ast::LoopOrSwitchLabel::Switch(switch_label) => &switch_label.identifier,
                }
            ),
        }
    }

    fn break_loop_label(label: &ast::LoopLabel) -> tacky::Label {
        Self::break_label(&ast::LoopOrSwitchLabel::Loop(label.clone()))
    }

    fn continue_label(label: &ast::LoopLabel) -> tacky::Label {
        tacky::Label {
            identifier: format!("{}.continue", label.identifier),
        }
    }

    fn handle_program(&mut self, program: &ast::Program, symbols: &SymbolTable) -> tacky::Program {
        let mut items = Vec::new();

        for declaration in &program.declarations {
            if let ast::Declaration::Function(fd) = declaration {
                if let Some(definition) = self.handle_top_level_function_declaration(fd, symbols) {
                    items.push(tacky::TopLevelItem::FunctionDefinition(definition));
                }
            }
        }

        for (identifier, symbol) in symbols.iter() {
            if let SymbolAttributes::Static { initial, global } = symbol.attrs {
                match initial {
                    SymbolInitialValue::Tentative => {
                        items.push(tacky::TopLevelItem::StaticVariable(tacky::StaticVariable {
                            variable: tacky::Variable {
                                identifier: identifier.clone(),
                            },
                            global,
                            initial: 0,
                        }));
                    }
                    SymbolInitialValue::Initial(initial) => {
                        items.push(tacky::TopLevelItem::StaticVariable(tacky::StaticVariable {
                            variable: tacky::Variable {
                                identifier: identifier.clone(),
                            },
                            global,
                            initial,
                        }));
                    }
                    SymbolInitialValue::None => {}
                }
            }
        }

        tacky::Program { items }
    }

    fn handle_top_level_function_declaration(
        &mut self,
        fd: &ast::FunctionDeclaration,
        symbols: &SymbolTable,
    ) -> Option<tacky::FunctionDefinition> {
        let Some(body) = &fd.body else {
            return None;
        };

        let mut instructions = self.handle_block(body);

        instructions.push(tacky::Instruction::Return(tacky::Value::Constant(0)));

        let symbol = symbols.get(&fd.function.identifier).unwrap();
        let SymbolAttributes::Function { global, .. } = symbol.attrs else {
            unreachable!()
        };

        Some(tacky::FunctionDefinition {
            function: tacky::Function {
                identifier: fd.function.identifier.clone(),
            },
            global,
            parameters: fd
                .parameters
                .iter()
                .cloned()
                .map(|p| tacky::Variable {
                    identifier: p.identifier,
                })
                .collect(),
            instructions,
        })
    }

    fn handle_block(&mut self, block: &ast::Block) -> Vec<tacky::Instruction> {
        let mut ins = vec![];

        for item in &block.items {
            match item {
                ast::BlockItem::Declaration(declaration) => {
                    self.handle_block_level_declaration(&mut ins, declaration);
                }
                ast::BlockItem::Statement(statement) => {
                    self.handle_statement(&mut ins, statement);
                }
            }
        }

        ins
    }

    fn handle_block_level_declaration(
        &mut self,
        ins: &mut Vec<tacky::Instruction>,
        declaration: &ast::Declaration,
    ) {
        match declaration {
            ast::Declaration::Variable(vd) => self.handle_block_level_variable_declaration(ins, vd),
            ast::Declaration::Function(_) => {}
        }
    }

    fn handle_block_level_variable_declaration(
        &mut self,
        ins: &mut Vec<tacky::Instruction>,
        vd: &ast::VariableDeclaration,
    ) {
        if vd.storage_class.is_some() {
            return;
        }

        if let Some(initializer) = &vd.initializer {
            let value = self.handle_expression(ins, initializer);

            ins.push(tacky::Instruction::Copy {
                src: value,
                dst: tacky::Variable {
                    identifier: vd.variable.identifier.clone(),
                },
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
            ast::Statement::Goto(label) => {
                ins.push(tacky::Instruction::Jump {
                    target: tacky::Label {
                        identifier: label.identifier.clone(),
                    },
                });
            }
            ast::Statement::Labeled(label, statement) => {
                ins.push(tacky::Instruction::Label(tacky::Label {
                    identifier: label.identifier.clone(),
                }));
                self.handle_statement(ins, statement);
            }
            ast::Statement::Compound(block) => {
                ins.extend(self.handle_block(block));
            }
            ast::Statement::Break(label) => {
                let Some(label) = label else {
                    unreachable!();
                };

                ins.push(tacky::Instruction::Jump {
                    target: Self::break_label(label),
                });
            }
            ast::Statement::Continue(label) => {
                let Some(label) = label else {
                    unreachable!();
                };

                ins.push(tacky::Instruction::Jump {
                    target: Self::continue_label(label),
                });
            }
            ast::Statement::While {
                condition,
                body,
                label,
            } => {
                let Some(label) = label else {
                    unreachable!();
                };

                ins.push(tacky::Instruction::Label(Self::continue_label(label)));
                let condition_value = self.handle_expression(ins, condition);
                ins.push(tacky::Instruction::JumpIfZero {
                    condition: condition_value,
                    target: Self::break_loop_label(label),
                });
                self.handle_statement(ins, body);
                ins.push(tacky::Instruction::Jump {
                    target: Self::continue_label(label),
                });
                ins.push(tacky::Instruction::Label(Self::break_loop_label(label)));
            }
            ast::Statement::DoWhile {
                body,
                condition,
                label,
            } => {
                let Some(label) = label else {
                    unreachable!();
                };
                let start_label = self.fresh_label(Some("do_while_start"));

                ins.push(tacky::Instruction::Label(start_label.clone()));
                self.handle_statement(ins, body);
                ins.push(tacky::Instruction::Label(Self::continue_label(label)));
                let condition_value = self.handle_expression(ins, condition);
                ins.push(tacky::Instruction::JumpIfNotZero {
                    condition: condition_value,
                    target: start_label.clone(),
                });
                ins.push(tacky::Instruction::Label(Self::break_loop_label(label)));
            }
            ast::Statement::For {
                initializer,
                condition,
                post,
                body,
                label,
            } => {
                let Some(label) = label else {
                    unreachable!();
                };
                let start_label = self.fresh_label(Some("for_start"));

                if let Some(initializer) = initializer {
                    match initializer {
                        ast::ForInitializer::VariableDeclaration(vd) => {
                            self.handle_block_level_variable_declaration(ins, vd);
                        }
                        ast::ForInitializer::Expression(expression) => {
                            self.handle_expression(ins, expression);
                        }
                    }
                }
                ins.push(tacky::Instruction::Label(start_label.clone()));
                if let Some(condition) = condition {
                    let condition_value = self.handle_expression(ins, condition);
                    ins.push(tacky::Instruction::JumpIfZero {
                        condition: condition_value,
                        target: Self::break_loop_label(label),
                    });
                }
                self.handle_statement(ins, body);
                ins.push(tacky::Instruction::Label(Self::continue_label(label)));
                if let Some(post) = post {
                    self.handle_expression(ins, post);
                }
                ins.push(tacky::Instruction::Jump {
                    target: start_label.clone(),
                });
                ins.push(tacky::Instruction::Label(Self::break_loop_label(label)));
            }
            ast::Statement::Null => {}
            ast::Statement::Switch { .. }
            | ast::Statement::Case { .. }
            | ast::Statement::Default { .. } => todo!(),
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
            ast::Expression::FunctionCall {
                function,
                arguments,
            } => {
                let dst = self.fresh_variable();

                let mut args = Vec::new();

                for arg in arguments {
                    args.push(self.handle_expression(ins, arg));
                }

                ins.push(tacky::Instruction::FunctionCall {
                    function: tacky::Function {
                        identifier: function.identifier.clone(),
                    },
                    args,
                    dst: dst.clone(),
                });

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
