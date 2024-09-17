use super::{
    ast::{BlockItem, Declaration, Expression, Function, Program, Statement, UnaryOperator},
    constants::SEMANTIC_VAR_PREFIX,
};
use std::collections::HashMap;

use super::ast::Variable;

struct VariableResolver {
    map: HashMap<String, String>,
    counter: usize,
}

impl VariableResolver {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            counter: 0,
        }
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
            body: self.handle_block_items(&function.body)?,
        })
    }

    fn handle_block_items(&mut self, body: &[BlockItem]) -> Result<Vec<BlockItem>, String> {
        let mut result = body.to_vec();
        for item in result.iter_mut() {
            match item {
                BlockItem::Declaration(declaration) => {
                    *item = BlockItem::Declaration(self.handle_declaration(declaration)?);
                }
                BlockItem::Statement(statement) => {
                    *item = BlockItem::Statement(self.handle_statement(statement)?);
                }
            }
        }
        Ok(result)
    }

    fn handle_declaration(&mut self, declaration: &Declaration) -> Result<Declaration, String> {
        if self.map.contains_key(&declaration.variable.identifier) {
            return Err(format!(
                "Variable {} already declared",
                declaration.variable.identifier
            ));
        }

        let new_variable = self.fresh_variable(Some(&declaration.variable.identifier));
        self.map.insert(
            declaration.variable.identifier.clone(),
            new_variable.identifier.clone(),
        );

        Ok(Declaration {
            variable: new_variable,
            initializer: declaration
                .initializer
                .as_ref()
                .map(|expr| self.handle_expression(expr))
                .transpose()?,
        })
    }

    fn handle_statement(&mut self, statement: &Statement) -> Result<Statement, String> {
        Ok(match statement {
            Statement::Return(expr) => Statement::Return(self.handle_expression(expr)?),
            Statement::Expression(expr) => Statement::Expression(self.handle_expression(expr)?),
            Statement::If { .. } => todo!(),
            Statement::Null => Statement::Null,
        })
    }

    fn handle_expression(&mut self, expr: &Expression) -> Result<Expression, String> {
        Ok(match expr {
            Expression::Constant(_) => expr.clone(),
            Expression::Variable(var) => {
                if let Some(new_name) = self.map.get(&var.identifier) {
                    Expression::Variable(Variable {
                        identifier: new_name.clone(),
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
                    expr: Box::new(self.handle_expression(expr)?),
                }
            }
            Expression::Binary { op, lhs, rhs } => Expression::Binary {
                op: *op,
                lhs: Box::new(self.handle_expression(lhs)?),
                rhs: Box::new(self.handle_expression(rhs)?),
            },
            Expression::Assignment { op, lhs, rhs } => {
                let Expression::Variable(_) = **lhs else {
                    return Err("Invalid lvalue in assignment".to_string());
                };
                Expression::Assignment {
                    op: *op,
                    lhs: Box::new(self.handle_expression(lhs)?),
                    rhs: Box::new(self.handle_expression(rhs)?),
                }
            }
            Expression::Conditional { .. } => todo!(),
        })
    }
}

pub fn analyze(program: &Program) -> Result<Program, String> {
    (VariableResolver::new()).handle_program(program)
}
