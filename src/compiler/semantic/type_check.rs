use crate::compiler::{
    ast::{
        Block, BlockItem, Declaration, Expression, ForInitializer, FunctionDeclaration, Program,
        Statement, VariableDeclaration,
    },
    types::Type,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum SymbolTableEntry {
    Variable { t: Type },
    Function { t: Type, defined: bool },
}

impl SymbolTableEntry {
    fn t(&self) -> &Type {
        match self {
            SymbolTableEntry::Variable { t } | SymbolTableEntry::Function { t, .. } => t,
        }
    }
}

pub struct SymbolTable {
    entries: HashMap<String, SymbolTableEntry>,
}

impl SymbolTable {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    fn get(&self, identifier: &str) -> Option<&SymbolTableEntry> {
        self.entries.get(identifier)
    }

    fn get_mut(&mut self, identifier: &str) -> Option<&mut SymbolTableEntry> {
        self.entries.get_mut(identifier)
    }

    fn insert(&mut self, identifier: String, entry: SymbolTableEntry) -> Option<SymbolTableEntry> {
        self.entries.insert(identifier, entry)
    }
}

pub struct TypeChecker {
    symbols: SymbolTable,
}

impl TypeChecker {
    fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
        }
    }

    pub fn analyze(program: &Program) -> Result<(Program, SymbolTable), String> {
        let mut tc = Self::new();

        let analyzed = tc.handle_program(program)?;

        Ok((analyzed, tc.symbols))
    }

    fn handle_program(&mut self, program: &Program) -> Result<Program, String> {
        todo!()

        // let mut function_declarations = Vec::new();

        // for fd in &program.function_declarations {
        //     function_declarations.push(self.handle_function_declaration(fd)?);
        // }

        // Ok(Program {
        //     function_declarations,
        // })
    }

    fn handle_function_declaration(
        &mut self,
        fd: &FunctionDeclaration,
    ) -> Result<FunctionDeclaration, String> {
        let t = Type::Function {
            parameter_count: fd.parameters.len(),
        };

        if let Some(entry) = self.symbols.get_mut(&fd.function.identifier) {
            let SymbolTableEntry::Function {
                t: entry_t,
                defined: entry_defined,
            } = entry
            else {
                unreachable!()
            };

            if *entry_t != t {
                return Err(format!(
                    "Incompatible redeclaration of function {}",
                    fd.function.identifier
                ));
            }

            if fd.body.is_some() {
                if *entry_defined {
                    return Err(format!(
                        "Redefinition of function {}",
                        fd.function.identifier
                    ));
                }

                *entry_defined = true;
            }
        } else {
            self.symbols.insert(
                fd.function.identifier.clone(),
                SymbolTableEntry::Function {
                    t,
                    defined: fd.body.is_some(),
                },
            );
        }

        let body = if let Some(body) = &fd.body {
            for parameter in &fd.parameters {
                self.symbols.insert(
                    parameter.identifier.clone(),
                    SymbolTableEntry::Variable { t: Type::Int },
                );
            }

            Some(self.handle_block(body)?)
        } else {
            None
        };

        todo!()

        // Ok(FunctionDeclaration {
        //     function: fd.function.clone(),
        //     parameters: fd.parameters.clone(),
        //     body,
        // })
    }

    fn handle_block(&mut self, block: &Block) -> Result<Block, String> {
        let mut result = block.clone();

        for item in result.items.iter_mut() {
            match item {
                BlockItem::Statement(statement) => {
                    *item = BlockItem::Statement(self.handle_statement(statement)?);
                }
                BlockItem::Declaration(declaration) => {
                    *item = BlockItem::Declaration(self.handle_declaration(declaration)?);
                }
            }
        }

        Ok(result)
    }

    fn handle_statement(&mut self, statement: &Statement) -> Result<Statement, String> {
        Ok(match statement {
            Statement::Return(expr) => Statement::Return(self.handle_expression(expr)?),
            Statement::Expression(expr) => Statement::Expression(self.handle_expression(expr)?),
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => Statement::If {
                condition: self.handle_expression(condition)?,
                then_branch: Box::new(self.handle_statement(then_branch)?),
                else_branch: if let Some(else_branch) = else_branch {
                    Some(Box::new(self.handle_statement(else_branch)?))
                } else {
                    None
                },
            },
            Statement::Labeled(label, statement) => {
                Statement::Labeled(label.clone(), Box::new(self.handle_statement(statement)?))
            }
            Statement::Compound(block) => Statement::Compound(self.handle_block(block)?),
            Statement::While {
                condition,
                body,
                label,
            } => Statement::While {
                condition: self.handle_expression(condition)?,
                body: Box::new(self.handle_statement(body)?),
                label: label.clone(),
            },
            Statement::DoWhile {
                body,
                condition,
                label,
            } => Statement::DoWhile {
                body: Box::new(self.handle_statement(body)?),
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
                    Some(ForInitializer::VariableDeclaration(vd)) => Some(
                        ForInitializer::VariableDeclaration(self.handle_variable_declaration(vd)?),
                    ),
                    Some(ForInitializer::Expression(expr)) => {
                        Some(ForInitializer::Expression(self.handle_expression(expr)?))
                    }
                    None => None,
                };

                let condition = self.handle_opt_expression(condition)?;
                let post = self.handle_opt_expression(post)?;
                let body = Box::new(self.handle_statement(body)?);

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

    fn handle_declaration(&mut self, declaration: &Declaration) -> Result<Declaration, String> {
        Ok(match declaration {
            Declaration::Variable(vd) => {
                Declaration::Variable(self.handle_variable_declaration(vd)?)
            }
            Declaration::Function(function_declaration) => {
                Declaration::Function(self.handle_function_declaration(function_declaration)?)
            }
        })
    }

    fn handle_variable_declaration(
        &mut self,
        vd: &VariableDeclaration,
    ) -> Result<VariableDeclaration, String> {
        self.symbols.insert(
            vd.variable.identifier.clone(),
            SymbolTableEntry::Variable { t: Type::Int },
        );

        let initializer = if let Some(expr) = &vd.initializer {
            Some(self.handle_expression(expr)?)
        } else {
            None
        };

        todo!()

        // Ok(VariableDeclaration {
        //     variable: vd.variable.clone(),
        //     initializer,
        // })
    }

    fn handle_expression(&mut self, expr: &Expression) -> Result<Expression, String> {
        Ok(match expr {
            Expression::FunctionCall {
                function,
                arguments,
            } => {
                let entry = self.symbols.get(&function.identifier).unwrap();

                let Type::Function { parameter_count } = entry.t() else {
                    return Err(format!("{} is not a function", function.identifier));
                };

                if *parameter_count != arguments.len() {
                    return Err(format!(
                        "Function {} expects {} arguments, got {}",
                        function.identifier,
                        parameter_count,
                        arguments.len()
                    ));
                }

                for argument in arguments {
                    self.handle_expression(argument)?;
                }

                expr.clone()
            }
            Expression::Variable(variable) => {
                let entry = self.symbols.get(&variable.identifier).unwrap();

                let Type::Int = entry.t() else {
                    return Err(format!("{} is not a variable", variable.identifier));
                };

                expr.clone()
            }
            Expression::Unary { op, expr } => Expression::Unary {
                op: *op,
                expr: Box::new(self.handle_expression(expr)?),
            },
            Expression::Binary { op, lhs, rhs } => Expression::Binary {
                op: *op,
                lhs: Box::new(self.handle_expression(lhs)?),
                rhs: Box::new(self.handle_expression(rhs)?),
            },
            Expression::Assignment { op, lhs, rhs } => Expression::Assignment {
                op: *op,
                lhs: Box::new(self.handle_expression(lhs)?),
                rhs: Box::new(self.handle_expression(rhs)?),
            },
            Expression::Conditional {
                condition,
                then_expr,
                else_expr,
            } => Expression::Conditional {
                condition: Box::new(self.handle_expression(condition)?),
                then_expr: Box::new(self.handle_expression(then_expr)?),
                else_expr: Box::new(self.handle_expression(else_expr)?),
            },
            Expression::Constant(_) => expr.clone(),
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
