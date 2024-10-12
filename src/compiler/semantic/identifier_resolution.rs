use crate::compiler::{
    ast::{
        Block, BlockItem, Declaration, Expression, ForInitializer, Function, FunctionDeclaration,
        Program, Statement, StorageClass, UnaryOperator, Variable, VariableDeclaration,
    },
    constants::SEMANTIC_VAR_PREFIX,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct IdentifierMapEntry {
    new_name: String,
    from_current_scope: bool,
    has_linkage: bool,
}

#[derive(Debug, Clone)]
struct IdentifierMap {
    map: HashMap<String, IdentifierMapEntry>,
}

impl IdentifierMap {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn get(&self, identifier: &str) -> Option<&IdentifierMapEntry> {
        self.map.get(identifier)
    }

    fn insert(
        &mut self,
        identifier: String,
        entry: IdentifierMapEntry,
    ) -> Option<IdentifierMapEntry> {
        self.map.insert(identifier, entry)
    }

    fn clone_rescoped(&self) -> Self {
        let mut clone = self.clone();

        for (_, entry) in clone.map.iter_mut() {
            entry.from_current_scope = false;
        }

        clone
    }
}

pub struct IdentifierResolver {
    counter: usize,
}

impl IdentifierResolver {
    fn new() -> Self {
        Self { counter: 0 }
    }

    pub fn analyze(program: &Program) -> Result<Program, String> {
        Self::new().handle_program(program)
    }

    fn fresh_variable(&mut self, suffix: Option<&str>) -> Variable {
        let name = match suffix {
            Some(suffix) => format!("{SEMANTIC_VAR_PREFIX}.{}.{}", self.counter, suffix),
            None => format!("{SEMANTIC_VAR_PREFIX}.{}", self.counter),
        };
        self.counter += 1;

        Variable { identifier: name }
    }

    fn handle_program(&mut self, program: &Program) -> Result<Program, String> {
        let mut map = IdentifierMap::new();
        let mut declarations = vec![];

        for declaration in &program.declarations {
            declarations.push(self.handle_top_level_declaration(declaration, &mut map)?);
        }

        Ok(Program { declarations })
    }

    fn handle_top_level_declaration(
        &mut self,
        declaration: &Declaration,
        map: &mut IdentifierMap,
    ) -> Result<Declaration, String> {
        Ok(match declaration {
            Declaration::Variable(vd) => {
                Declaration::Variable(self.handle_top_level_variable_declaration(vd, map)?)
            }
            Declaration::Function(fd) => {
                Declaration::Function(self.handle_top_level_function_declaration(fd, map)?)
            }
        })
    }

    fn handle_top_level_variable_declaration(
        &mut self,
        declaration: &VariableDeclaration,
        map: &mut IdentifierMap,
    ) -> Result<VariableDeclaration, String> {
        map.insert(
            declaration.variable.identifier.clone(),
            IdentifierMapEntry {
                new_name: declaration.variable.identifier.clone(),
                from_current_scope: true,
                has_linkage: true,
            },
        );

        Ok(declaration.clone())
    }

    fn handle_top_level_function_declaration(
        &mut self,
        declaration: &FunctionDeclaration,
        map: &mut IdentifierMap,
    ) -> Result<FunctionDeclaration, String> {
        if let Some(entry) = map.get(&declaration.function.identifier) {
            if entry.from_current_scope && !entry.has_linkage {
                return Err(format!(
                    "Duplicate declaration of identifier {}",
                    declaration.function.identifier
                ));
            }
        }

        map.insert(
            declaration.function.identifier.clone(),
            IdentifierMapEntry {
                new_name: declaration.function.identifier.clone(),
                from_current_scope: true,
                has_linkage: true,
            },
        );

        let mut inner_map = map.clone_rescoped();

        let mut parameters = Vec::new();
        for parameter in &declaration.parameters {
            parameters.push(self.handle_parameter(parameter, &mut inner_map)?);
        }

        let body = if let Some(body) = declaration.body.clone() {
            Some(self.handle_block(&body, inner_map)?)
        } else {
            None
        };

        Ok(FunctionDeclaration {
            function: declaration.function.clone(),
            parameters,
            body,
            storage_class: declaration.storage_class,
        })
    }

    fn handle_parameter(
        &mut self,
        parameter: &Variable,
        map: &mut IdentifierMap,
    ) -> Result<Variable, String> {
        if let Some(entry) = map.get(&parameter.identifier) {
            if entry.from_current_scope {
                return Err(format!(
                    "Duplicate declaration of identifier {}",
                    parameter.identifier,
                ));
            }
        }

        let fresh = self.fresh_variable(Some(&parameter.identifier));
        map.insert(
            parameter.identifier.clone(),
            IdentifierMapEntry {
                new_name: fresh.identifier.clone(),
                from_current_scope: true,
                has_linkage: false,
            },
        );

        Ok(fresh)
    }

    fn handle_block(
        &mut self,
        block: &Block,
        mut inner_map: IdentifierMap,
    ) -> Result<Block, String> {
        let mut result = block.clone();
        for item in result.items.iter_mut() {
            match item {
                BlockItem::Declaration(declaration) => {
                    *item = BlockItem::Declaration(
                        self.handle_block_level_declaration(declaration, &mut inner_map)?,
                    );
                }
                BlockItem::Statement(statement) => {
                    *item = BlockItem::Statement(self.handle_statement(statement, &inner_map)?);
                }
            }
        }
        Ok(result)
    }

    fn handle_block_level_declaration(
        &mut self,
        declaration: &Declaration,
        map: &mut IdentifierMap,
    ) -> Result<Declaration, String> {
        Ok(match declaration {
            Declaration::Variable(vd) => {
                Declaration::Variable(self.handle_block_level_variable_declaration(vd, map)?)
            }
            Declaration::Function(fd) => {
                Declaration::Function(self.handle_block_level_function_declaration(fd, map)?)
            }
        })
    }

    fn handle_block_level_variable_declaration(
        &mut self,
        declaration: &VariableDeclaration,
        map: &mut IdentifierMap,
    ) -> Result<VariableDeclaration, String> {
        if let Some(entry) = map.get(&declaration.variable.identifier) {
            if entry.from_current_scope
                && !(entry.has_linkage && declaration.storage_class == Some(StorageClass::Extern))
            {
                return Err(format!(
                    "Conflicting block-level declarations of identifier {}",
                    declaration.variable.identifier
                ));
            }
        }

        if declaration.storage_class == Some(StorageClass::Extern) {
            map.insert(
                declaration.variable.identifier.clone(),
                IdentifierMapEntry {
                    new_name: declaration.variable.identifier.clone(),
                    from_current_scope: true,
                    has_linkage: true,
                },
            );

            Ok(declaration.clone())
        } else {
            let fresh = self.fresh_variable(Some(&declaration.variable.identifier));
            map.insert(
                declaration.variable.identifier.clone(),
                IdentifierMapEntry {
                    new_name: fresh.identifier.clone(),
                    from_current_scope: true,
                    has_linkage: false,
                },
            );

            let initializer = if let Some(initializer) = &declaration.initializer {
                Some(Self::handle_expression(initializer, map)?)
            } else {
                None
            };

            Ok(VariableDeclaration {
                variable: fresh,
                initializer,
                storage_class: declaration.storage_class,
            })
        }
    }

    fn handle_block_level_function_declaration(
        &mut self,
        declaration: &FunctionDeclaration,
        map: &mut IdentifierMap,
    ) -> Result<FunctionDeclaration, String> {
        if declaration.body.is_some() {
            return Err("Block level function declarations cannot have bodies".to_string());
        }

        if declaration.storage_class == Some(StorageClass::Static) {
            return Err(
                "Block level function declarations cannot have static storage class specifiers"
                    .to_string(),
            );
        }

        self.handle_top_level_function_declaration(declaration, map)
    }

    fn handle_statement(
        &mut self,
        statement: &Statement,
        map: &IdentifierMap,
    ) -> Result<Statement, String> {
        Ok(match statement {
            Statement::Return(expr) => Statement::Return(Self::handle_expression(expr, map)?),
            Statement::Expression(expr) => {
                Statement::Expression(Self::handle_expression(expr, map)?)
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => Statement::If {
                condition: Self::handle_expression(condition, map)?,
                then_branch: Box::new(self.handle_statement(then_branch, map)?),
                else_branch: if let Some(else_branch) = else_branch {
                    Some(Box::new(self.handle_statement(else_branch, map)?))
                } else {
                    None
                },
            },
            Statement::Labeled(label, statement) => Statement::Labeled(
                label.clone(),
                Box::new(self.handle_statement(statement, map)?),
            ),
            Statement::Compound(block) => {
                Statement::Compound(self.handle_block(block, map.clone_rescoped())?)
            }
            Statement::While {
                condition,
                body,
                label,
            } => Statement::While {
                condition: Self::handle_expression(condition, map)?,
                body: Box::new(self.handle_statement(body, map)?),
                label: label.clone(),
            },
            Statement::DoWhile {
                body,
                condition,
                label,
            } => Statement::DoWhile {
                body: Box::new(self.handle_statement(body, map)?),
                condition: Self::handle_expression(condition, map)?,
                label: label.clone(),
            },
            Statement::For {
                initializer,
                condition,
                post,
                body,
                label,
            } => {
                let mut inner_map = map.clone_rescoped();

                let initializer = match initializer {
                    Some(ForInitializer::VariableDeclaration(declaration)) => {
                        Some(ForInitializer::VariableDeclaration(
                            self.handle_block_level_variable_declaration(
                                declaration,
                                &mut inner_map,
                            )?,
                        ))
                    }
                    Some(ForInitializer::Expression(expr)) => Some(ForInitializer::Expression(
                        Self::handle_expression(expr, map)?,
                    )),
                    None => None,
                };

                let condition = Self::handle_opt_expression(condition, &inner_map)?;
                let post = Self::handle_opt_expression(post, &inner_map)?;
                let body = Box::new(self.handle_statement(body, &inner_map)?);

                Statement::For {
                    initializer,
                    condition,
                    post,
                    body,
                    label: label.clone(),
                }
            }
            Statement::Null | Statement::Goto(_) | Statement::Break(_) | Statement::Continue(_) => {
                statement.clone()
            }
        })
    }

    fn handle_expression(expr: &Expression, map: &IdentifierMap) -> Result<Expression, String> {
        Ok(match expr {
            Expression::Constant(_) => expr.clone(),
            Expression::Variable(var) => {
                if let Some(entry) = map.get(&var.identifier) {
                    Expression::Variable(Variable {
                        identifier: entry.new_name.clone(),
                    })
                } else {
                    return Err(format!("Variable {} not declared", var.identifier));
                }
            }
            Expression::Unary { op, expr } => {
                if let UnaryOperator::PrefixIncrement
                | UnaryOperator::PrefixDecrement
                | UnaryOperator::PostfixIncrement
                | UnaryOperator::PostfixDecrement = *op
                {
                    let Expression::Variable(_) = **expr else {
                        return Err("Invalid lvalue in increment/decrement".to_string());
                    };
                }
                Expression::Unary {
                    op: *op,
                    expr: Box::new(Self::handle_expression(expr, map)?),
                }
            }
            Expression::Binary { op, lhs, rhs } => Expression::Binary {
                op: *op,
                lhs: Box::new(Self::handle_expression(lhs, map)?),
                rhs: Box::new(Self::handle_expression(rhs, map)?),
            },
            Expression::Assignment { op, lhs, rhs } => {
                let Expression::Variable(_) = **lhs else {
                    return Err("Invalid lvalue in assignment".to_string());
                };
                Expression::Assignment {
                    op: *op,
                    lhs: Box::new(Self::handle_expression(lhs, map)?),
                    rhs: Box::new(Self::handle_expression(rhs, map)?),
                }
            }
            Expression::Conditional {
                condition,
                then_expr,
                else_expr,
            } => Expression::Conditional {
                condition: Box::new(Self::handle_expression(condition, map)?),
                then_expr: Box::new(Self::handle_expression(then_expr, map)?),
                else_expr: Box::new(Self::handle_expression(else_expr, map)?),
            },
            Expression::FunctionCall {
                function,
                arguments,
            } => {
                if let Some(entry) = map.get(&function.identifier) {
                    let new_name = entry.new_name.clone();
                    let mut new_arguments = Vec::new();

                    for argument in arguments {
                        new_arguments.push(Self::handle_expression(argument, map)?);
                    }

                    Expression::FunctionCall {
                        function: Function {
                            identifier: new_name,
                        },
                        arguments: new_arguments,
                    }
                } else {
                    return Err(format!("Function {} not declared", function.identifier));
                }
            }
        })
    }

    fn handle_opt_expression(
        opt_expr: &Option<Expression>,
        map: &IdentifierMap,
    ) -> Result<Option<Expression>, String> {
        Ok(if let Some(expr) = opt_expr {
            Some(Self::handle_expression(expr, map)?)
        } else {
            None
        })
    }
}
