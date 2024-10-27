use crate::compiler::{
    ast::{
        BinaryOperator, Block, BlockItem, Constant, Declaration, Expression, ForInitializer,
        FunctionDeclaration, Program, Statement, StorageClass, Type, UnaryOperator::Not,
        VariableDeclaration,
    },
    symbols::{Symbol, SymbolAttributes, SymbolInitialValue, SymbolStaticInitial, SymbolTable},
};

pub struct TypeChecker {
    symbols: SymbolTable,
}

impl TypeChecker {
    pub fn analyze(program: &Program) -> Result<(Program, SymbolTable), String> {
        let mut tc = Self::new();

        let analyzed = tc.handle_program(program)?;

        Ok((analyzed, tc.symbols))
    }

    fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
        }
    }

    fn get_common_type(&self, ty1: &Type, ty2: &Type) -> Type {
        if ty1 == ty2 {
            ty1.clone()
        } else {
            Type::Long
        }
    }

    fn convert_to_type(&self, expr: &Expression, ty: &Type) -> Expression {
        if expr.ty().unwrap() == *ty {
            expr.clone()
        } else {
            Expression::Cast {
                target_ty: ty.clone(),
                expr: Box::new(expr.clone()),
                ty: Some(ty.clone()),
            }
        }
    }

    fn constant_to_static_initial(&self, c: &Constant, ty: &Type) -> SymbolStaticInitial {
        match ty {
            Type::Int => SymbolStaticInitial::Int(match c {
                Constant::ConstantInt(n) => *n,
                Constant::ConstantLong(n) => *n as i32,
            }),
            Type::Long => SymbolStaticInitial::Long(match c {
                Constant::ConstantInt(n) => *n as i64,
                Constant::ConstantLong(n) => *n,
            }),
            Type::Function { .. } => unreachable!(),
        }
    }

    fn handle_program(&mut self, program: &Program) -> Result<Program, String> {
        let mut declarations = Vec::new();

        for declaration in &program.declarations {
            declarations.push(self.handle_top_level_declaration(declaration)?);
        }

        Ok(Program { declarations })
    }

    fn handle_top_level_declaration(
        &mut self,
        declaration: &Declaration,
    ) -> Result<Declaration, String> {
        Ok(match declaration {
            Declaration::Variable(vd) => {
                Declaration::Variable(self.handle_top_level_variable_declaration(vd)?)
            }
            Declaration::Function(fd) => {
                Declaration::Function(self.handle_function_declaration(fd)?)
            }
        })
    }

    fn handle_top_level_variable_declaration(
        &mut self,
        declaration: &VariableDeclaration,
    ) -> Result<VariableDeclaration, String> {
        let mut initial = match &declaration.initializer {
            Some(Expression::Constant { c, ty: _ }) => {
                SymbolInitialValue::Initial(self.constant_to_static_initial(c, &declaration.ty))
            }
            None => {
                if declaration.storage_class == Some(StorageClass::Extern) {
                    SymbolInitialValue::None
                } else {
                    SymbolInitialValue::Tentative
                }
            }
            _ => return Err("Non-constant initializer".to_string()),
        };

        let mut global = declaration.storage_class != Some(StorageClass::Static);

        if let Some(entry) = self.symbols.get(&declaration.variable.identifier) {
            if entry.ty != declaration.ty {
                return Err(format!(
                    "Incompatible redeclaration of variable {}",
                    declaration.variable.identifier
                ));
            }

            let SymbolAttributes::Static {
                initial: entry_initial,
                global: entry_global,
            } = entry.attrs
            else {
                unreachable!()
            };

            if declaration.storage_class == Some(StorageClass::Extern) {
                global = entry_global;
            } else if entry_global != global {
                return Err(format!(
                    "Conflicting variable linkage of {}",
                    declaration.variable.identifier
                ));
            }

            match entry_initial {
                SymbolInitialValue::Initial(_) => {
                    if let SymbolInitialValue::Initial(_) = initial {
                        return Err(format!(
                            "Conflicting file scope variable definition of {}",
                            declaration.variable.identifier
                        ));
                    }

                    initial = entry_initial;
                }
                SymbolInitialValue::Tentative => {
                    if !matches!(initial, SymbolInitialValue::Initial(_)) {
                        initial = SymbolInitialValue::Tentative;
                    }
                }
                _ => {}
            };
        }

        self.symbols.insert(
            declaration.variable.identifier.clone(),
            Symbol {
                ty: declaration.ty.clone(),
                attrs: SymbolAttributes::Static { initial, global },
            },
        );

        Ok(declaration.clone())
    }

    fn handle_function_declaration(
        &mut self,
        declaration: &FunctionDeclaration,
    ) -> Result<FunctionDeclaration, String> {
        let Type::Function {
            return_type,
            parameters,
        } = &declaration.ty
        else {
            unreachable!()
        };

        let has_body = declaration.body.is_some();
        let mut already_defined = false;
        let mut global = declaration.storage_class != Some(StorageClass::Static);

        if let Some(entry) = self.symbols.get(&declaration.function.identifier) {
            if entry.ty != declaration.ty {
                return Err(format!(
                    "Incompatible redeclaration of function {}",
                    declaration.function.identifier
                ));
            }

            let SymbolAttributes::Function {
                defined: entry_defined,
                global: entry_global,
            } = entry.attrs
            else {
                unreachable!()
            };

            already_defined = entry_defined;

            if already_defined && has_body {
                return Err(format!(
                    "Redefinition of function {}",
                    declaration.function.identifier
                ));
            }

            if entry_global && declaration.storage_class == Some(StorageClass::Static) {
                return Err(format!(
                    "Static function declaration of {} after non-static declaration",
                    declaration.function.identifier
                ));
            }

            global = entry_global;
        }

        self.symbols.insert(
            declaration.function.identifier.clone(),
            Symbol {
                ty: declaration.ty.clone(),
                attrs: SymbolAttributes::Function {
                    defined: already_defined || has_body,
                    global,
                },
            },
        );

        let body = if let Some(body) = &declaration.body {
            for (parameter, parameter_ty) in declaration.parameters.iter().zip(parameters.iter()) {
                self.symbols.insert(
                    parameter.identifier.clone(),
                    Symbol {
                        ty: parameter_ty.clone(),
                        attrs: SymbolAttributes::Local,
                    },
                );
            }

            Some(self.handle_block(body, return_type)?)
        } else {
            None
        };

        Ok(FunctionDeclaration {
            function: declaration.function.clone(),
            parameters: declaration.parameters.clone(),
            body,
            ty: declaration.ty.clone(),
            storage_class: declaration.storage_class,
        })
    }

    fn handle_block(
        &mut self,
        block: &Block,
        enclosing_return_type: &Type,
    ) -> Result<Block, String> {
        let mut result = block.clone();

        for item in result.items.iter_mut() {
            match item {
                BlockItem::Statement(statement) => {
                    *item = BlockItem::Statement(
                        self.handle_statement(statement, enclosing_return_type)?,
                    );
                }
                BlockItem::Declaration(declaration) => {
                    *item =
                        BlockItem::Declaration(self.handle_block_level_declaration(declaration)?);
                }
            }
        }

        Ok(result)
    }

    fn handle_statement(
        &mut self,
        statement: &Statement,
        enclosing_return_type: &Type,
    ) -> Result<Statement, String> {
        Ok(match statement {
            Statement::Return(expr) => {
                let typed_expr = self.handle_expression(expr)?;
                let converted_expr = self.convert_to_type(&typed_expr, enclosing_return_type);

                Statement::Return(converted_expr)
            }
            Statement::Expression(expr) => Statement::Expression(self.handle_expression(expr)?),
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => Statement::If {
                condition: self.handle_expression(condition)?,
                then_branch: Box::new(self.handle_statement(then_branch, enclosing_return_type)?),
                else_branch: if let Some(else_branch) = else_branch {
                    Some(Box::new(
                        self.handle_statement(else_branch, enclosing_return_type)?,
                    ))
                } else {
                    None
                },
            },
            Statement::Labeled(label, statement) => Statement::Labeled(
                label.clone(),
                Box::new(self.handle_statement(statement, enclosing_return_type)?),
            ),
            Statement::Compound(block) => {
                Statement::Compound(self.handle_block(block, enclosing_return_type)?)
            }
            Statement::While {
                condition,
                body,
                label,
            } => Statement::While {
                condition: self.handle_expression(condition)?,
                body: Box::new(self.handle_statement(body, enclosing_return_type)?),
                label: label.clone(),
            },
            Statement::DoWhile {
                body,
                condition,
                label,
            } => Statement::DoWhile {
                body: Box::new(self.handle_statement(body, enclosing_return_type)?),
                condition: self.handle_expression(condition)?,
                label: label.clone(),
            },
            Statement::For {
                initializer,
                condition,
                post,
                body,
                label,
            } => {
                let initializer = match initializer {
                    Some(ForInitializer::VariableDeclaration(vd)) => {
                        if vd.storage_class.is_some() {
                            return Err("For loop variable declaration cannot have storage class"
                                .to_string());
                        }

                        Some(ForInitializer::VariableDeclaration(
                            self.handle_block_level_variable_declaration(vd)?,
                        ))
                    }
                    Some(ForInitializer::Expression(expr)) => {
                        Some(ForInitializer::Expression(self.handle_expression(expr)?))
                    }
                    None => None,
                };

                let condition = self.handle_opt_expression(condition)?;
                let post = self.handle_opt_expression(post)?;
                let body = Box::new(self.handle_statement(body, enclosing_return_type)?);

                Statement::For {
                    initializer,
                    condition,
                    post,
                    body,
                    label: label.clone(),
                }
            }
            Statement::Switch {
                expression,
                body,
                cases,
                label,
            } => {
                let expression = self.handle_expression(expression)?;
                let body = Box::new(self.handle_statement(body, enclosing_return_type)?);

                Statement::Switch {
                    expression,
                    body,
                    cases: cases.clone(),
                    label: label.clone(),
                }
            }
            Statement::Case {
                expression,
                body,
                label,
            } => Statement::Case {
                expression: expression.clone(),
                body: Box::new(self.handle_statement(body, enclosing_return_type)?),
                label: label.clone(),
            },
            Statement::Default { body, label } => Statement::Default {
                body: Box::new(self.handle_statement(body, enclosing_return_type)?),
                label: label.clone(),
            },

            Statement::Null | Statement::Goto(_) | Statement::Break(_) | Statement::Continue(_) => {
                statement.clone()
            }
        })
    }

    fn handle_block_level_declaration(
        &mut self,
        declaration: &Declaration,
    ) -> Result<Declaration, String> {
        Ok(match declaration {
            Declaration::Variable(vd) => {
                Declaration::Variable(self.handle_block_level_variable_declaration(vd)?)
            }
            Declaration::Function(function_declaration) => {
                Declaration::Function(self.handle_function_declaration(function_declaration)?)
            }
        })
    }

    fn handle_block_level_variable_declaration(
        &mut self,
        declaration: &VariableDeclaration,
    ) -> Result<VariableDeclaration, String> {
        Ok(match declaration.storage_class {
            Some(StorageClass::Extern) => {
                if declaration.initializer.is_some() {
                    return Err(
                        "Block-level extern variable cannot have an initializer".to_string()
                    );
                }

                if let Some(entry) = self.symbols.get(&declaration.variable.identifier) {
                    if entry.ty != declaration.ty {
                        return Err(format!(
                            "Incompatible redeclaration of variable {}",
                            declaration.variable.identifier
                        ));
                    }
                } else {
                    self.symbols.insert(
                        declaration.variable.identifier.clone(),
                        Symbol {
                            ty: declaration.ty.clone(),
                            attrs: SymbolAttributes::Static {
                                initial: SymbolInitialValue::None,
                                global: true,
                            },
                        },
                    );
                }

                declaration.clone()
            }
            Some(StorageClass::Static) => {
                let initial = match &declaration.initializer {
                    Some(Expression::Constant { c, ty: _ }) => SymbolInitialValue::Initial(
                        self.constant_to_static_initial(&c, &declaration.ty),
                    ),
                    None => SymbolInitialValue::Initial(
                        self.constant_to_static_initial(&Constant::ConstantInt(0), &declaration.ty),
                    ),
                    _ => {
                        return Err(
                            "Non-constant initializer on block-level static variable".to_string()
                        )
                    }
                };

                self.symbols.insert(
                    declaration.variable.identifier.clone(),
                    Symbol {
                        ty: declaration.ty.clone(),
                        attrs: SymbolAttributes::Static {
                            initial,
                            global: false,
                        },
                    },
                );

                declaration.clone()
            }
            None => {
                self.symbols.insert(
                    declaration.variable.identifier.clone(),
                    Symbol {
                        ty: declaration.ty.clone(),
                        attrs: SymbolAttributes::Local,
                    },
                );

                let initializer = if let Some(expr) = &declaration.initializer {
                    let typed = self.handle_expression(expr)?;
                    let converted = self.convert_to_type(&typed, &declaration.ty);
                    Some(converted)
                } else {
                    None
                };

                VariableDeclaration {
                    variable: declaration.variable.clone(),
                    initializer,
                    ty: declaration.ty.clone(),
                    storage_class: declaration.storage_class,
                }
            }
        })
    }

    fn handle_expression(&mut self, expr: &Expression) -> Result<Expression, String> {
        Ok(match expr {
            Expression::FunctionCall {
                function,
                arguments,
                ty: _,
            } => {
                let entry = self.symbols.get(&function.identifier).unwrap().clone();

                let Type::Function {
                    return_type,
                    parameters,
                } = entry.ty
                else {
                    return Err(format!("{} is not a function", function.identifier));
                };

                if parameters.len() != arguments.len() {
                    return Err(format!(
                        "Function {} expects {} arguments, got {}",
                        function.identifier,
                        parameters.len(),
                        arguments.len()
                    ));
                }

                let mut converted_arguments = Vec::new();

                for (argument, parameter_ty) in arguments.iter().zip(parameters.iter()) {
                    let typed = self.handle_expression(argument)?;

                    converted_arguments.push(self.convert_to_type(&typed, parameter_ty));
                }

                Expression::FunctionCall {
                    function: function.clone(),
                    arguments: converted_arguments,
                    ty: Some(*return_type.clone()),
                }
            }
            Expression::Variable { v, ty: _ } => {
                let entry = self.symbols.get(&v.identifier).unwrap();

                if let Type::Function { .. } = entry.ty {
                    return Err(format!("{} is not a variable", v.identifier));
                }

                Expression::Variable {
                    v: v.clone(),
                    ty: Some(entry.ty.clone()),
                }
            }
            Expression::Unary { op, expr, ty: _ } => {
                let typed = self.handle_expression(expr)?;
                let ty = typed.ty().unwrap();

                Expression::Unary {
                    op: *op,
                    expr: Box::new(typed),
                    ty: Some(match op {
                        Not => Type::Int,
                        _ => ty,
                    }),
                }
            }
            Expression::Binary {
                op,
                lhs,
                rhs,
                ty: _,
            } => {
                let typed_lhs = self.handle_expression(lhs)?;
                let typed_rhs = self.handle_expression(rhs)?;

                if let BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr = op {
                    Expression::Binary {
                        op: *op,
                        lhs: Box::new(typed_lhs),
                        rhs: Box::new(typed_rhs),
                        ty: Some(Type::Int),
                    }
                } else {
                    let ty_lhs = typed_lhs.ty().unwrap();
                    let ty_rhs = typed_rhs.ty().unwrap();

                    let common = self.get_common_type(&ty_lhs, &ty_rhs);

                    let converted_lhs = self.convert_to_type(&typed_lhs, &common);
                    let converted_rhs = self.convert_to_type(&typed_rhs, &common);

                    let ty = match op {
                        BinaryOperator::Add
                        | BinaryOperator::Subtract
                        | BinaryOperator::Multiply
                        | BinaryOperator::Divide
                        | BinaryOperator::Remainder
                        | BinaryOperator::BitwiseAnd
                        | BinaryOperator::BitwiseOr
                        | BinaryOperator::BitwiseXor
                        | BinaryOperator::ShiftLeft
                        | BinaryOperator::ShiftRight => common,
                        BinaryOperator::Equal
                        | BinaryOperator::NotEqual
                        | BinaryOperator::LessThan
                        | BinaryOperator::LessOrEqual
                        | BinaryOperator::GreaterThan
                        | BinaryOperator::GreaterOrEqual => Type::Int,
                        BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => unreachable!(),
                    };

                    Expression::Binary {
                        op: *op,
                        lhs: Box::new(converted_lhs),
                        rhs: Box::new(converted_rhs),
                        ty: Some(ty),
                    }
                }
            }
            Expression::Assignment {
                op,
                lhs,
                rhs,
                ty: _,
            } => {
                let typed_lhs = self.handle_expression(lhs)?;
                let typed_rhs = self.handle_expression(rhs)?;

                let ty_lhs = typed_lhs.ty().unwrap();

                let converted_rhs = self.convert_to_type(&typed_rhs, &ty_lhs);

                Expression::Assignment {
                    op: *op,
                    lhs: Box::new(typed_lhs),
                    rhs: Box::new(converted_rhs),
                    ty: Some(ty_lhs),
                }
            }
            Expression::Conditional {
                condition,
                then_expr,
                else_expr,
                ty: _,
            } => {
                let typed_condition = self.handle_expression(condition)?;
                let typed_then = self.handle_expression(then_expr)?;
                let typed_else = self.handle_expression(else_expr)?;

                let ty_then = typed_then.ty().unwrap();
                let ty_else = typed_else.ty().unwrap();

                let common = self.get_common_type(&ty_then, &ty_else);

                let converted_then = self.convert_to_type(&typed_then, &common);
                let converted_else = self.convert_to_type(&typed_else, &common);

                Expression::Conditional {
                    condition: Box::new(typed_condition),
                    then_expr: Box::new(converted_then),
                    else_expr: Box::new(converted_else),
                    ty: Some(common),
                }
            }
            Expression::Constant { c, ty: _ } => Expression::Constant {
                c: c.clone(),
                ty: Some(match c {
                    Constant::ConstantInt(_) => Type::Int,
                    Constant::ConstantLong(_) => Type::Long,
                }),
            },
            Expression::Cast {
                target_ty,
                expr,
                ty: _,
            } => Expression::Cast {
                target_ty: target_ty.clone(),
                expr: Box::new(self.handle_expression(expr)?),
                ty: Some(target_ty.clone()),
            },
        })
    }

    fn handle_opt_expression(
        &mut self,
        expr: &Option<Expression>,
    ) -> Result<Option<Expression>, String> {
        Ok(match expr {
            Some(expr) => Some(self.handle_expression(expr)?),
            None => None,
        })
    }
}
