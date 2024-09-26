use crate::compiler::{
    ast::{
        Block, BlockItem, Declaration, Expression, ForInitializer, Function, Program, Statement,
        UnaryOperator, Variable,
    },
    constants::SEMANTIC_VAR_PREFIX,
};
use std::collections::HashMap;

type VariableMap = HashMap<String, String>;

pub struct VariableResolver {
    counter: usize,
}

impl VariableResolver {
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
        Ok(Program {
            function_definition: self.handle_function(&program.function_definition)?,
        })
    }

    fn handle_function(&mut self, function: &Function) -> Result<Function, String> {
        Ok(Function {
            name: function.name.clone(),
            body: self.handle_block(&function.body, &HashMap::new())?,
        })
    }

    fn handle_block(&mut self, block: &Block, outer_map: &VariableMap) -> Result<Block, String> {
        let mut result = block.clone();
        let mut inner_map = VariableMap::new();
        for item in result.items.iter_mut() {
            match item {
                BlockItem::Declaration(declaration) => {
                    *item = BlockItem::Declaration(self.handle_declaration(
                        declaration,
                        outer_map,
                        &mut inner_map,
                    )?);
                }
                BlockItem::Statement(statement) => {
                    let mut merged = outer_map.clone();
                    merged.extend(inner_map.clone());
                    *item = BlockItem::Statement(self.handle_statement(statement, &merged)?);
                }
            }
        }
        Ok(result)
    }

    fn handle_declaration(
        &mut self,
        declaration: &Declaration,
        outer_map: &VariableMap,
        inner_map: &mut VariableMap,
    ) -> Result<Declaration, String> {
        if inner_map.contains_key(&declaration.variable.identifier) {
            return Err(format!(
                "Variable {} already declared",
                declaration.variable.identifier
            ));
        }

        let new_variable = self.fresh_variable(Some(&declaration.variable.identifier));
        inner_map.insert(
            declaration.variable.identifier.clone(),
            new_variable.identifier.clone(),
        );

        let mut merged_map = outer_map.clone();
        merged_map.extend(inner_map.clone());

        Ok(Declaration {
            variable: new_variable,
            initializer: declaration
                .initializer
                .as_ref()
                .map(|expr| Self::handle_expression(expr, &merged_map))
                .transpose()?,
        })
    }

    fn handle_statement(
        &mut self,
        statement: &Statement,
        map: &VariableMap,
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
            Statement::Compound(block) => Statement::Compound(self.handle_block(block, map)?),
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
                let mut inner_map = VariableMap::new();

                let initializer = match initializer {
                    Some(ForInitializer::Declaration(declaration)) => {
                        let declaration =
                            self.handle_declaration(declaration, map, &mut inner_map)?;
                        Some(ForInitializer::Declaration(declaration))
                    }
                    Some(ForInitializer::Expression(expr)) => Some(ForInitializer::Expression(
                        Self::handle_expression(expr, map)?,
                    )),
                    None => None,
                };

                let mut merged_map = map.clone();
                merged_map.extend(inner_map.clone());

                let condition = Self::handle_opt_expression(condition, &merged_map)?;
                let post = Self::handle_opt_expression(post, &merged_map)?;
                let body = Box::new(self.handle_statement(body, &merged_map)?);

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

    fn handle_expression(expr: &Expression, map: &VariableMap) -> Result<Expression, String> {
        Ok(match expr {
            Expression::Constant(_) => expr.clone(),
            Expression::Variable(var) => {
                if let Some(new_identifier) = map.get(&var.identifier) {
                    Expression::Variable(Variable {
                        identifier: new_identifier.clone(),
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
        })
    }

    fn handle_opt_expression(
        opt_expr: &Option<Expression>,
        map: &VariableMap,
    ) -> Result<Option<Expression>, String> {
        if let Some(expr) = opt_expr {
            Ok(Some(Self::handle_expression(expr, map)?))
        } else {
            Ok(None)
        }
    }
}
