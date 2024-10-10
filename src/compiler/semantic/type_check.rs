use crate::compiler::{
    ast::{
        Block, BlockItem, Declaration, Expression, ForInitializer, FunctionDeclaration, Program,
        Statement, StorageClass, VariableDeclaration,
    },
    types::Type,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum SymbolAttributes {
    Function {
        defined: bool,
        global: bool,
    },
    Static {
        initial: SymbolInitialValue,
        global: bool,
    },
    Local,
}

#[derive(Debug, Clone, Copy)]
enum SymbolInitialValue {
    Tentative,
    Initial(i64),
    None,
}

#[derive(Debug, Clone)]
struct Symbol {
    t: Type,
    attrs: SymbolAttributes,
}

pub struct SymbolTable {
    entries: HashMap<String, Symbol>,
}

impl SymbolTable {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    fn get(&self, identifier: &str) -> Option<&Symbol> {
        self.entries.get(identifier)
    }

    fn insert(&mut self, identifier: String, entry: Symbol) -> Option<Symbol> {
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
        let t = Type::Int;

        let mut initial = match declaration.initializer {
            Some(Expression::Constant(i)) => SymbolInitialValue::Initial(i),
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
            if entry.t != t {
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
                t: Type::Int,
                attrs: SymbolAttributes::Static { initial, global },
            },
        );

        Ok(declaration.clone())
    }

    fn handle_function_declaration(
        &mut self,
        declaration: &FunctionDeclaration,
    ) -> Result<FunctionDeclaration, String> {
        let t = Type::Function {
            parameter_count: declaration.parameters.len(),
        };

        let has_body = declaration.body.is_some();
        let mut already_defined = false;
        let mut global = declaration.storage_class != Some(StorageClass::Static);

        if let Some(entry) = self.symbols.get(&declaration.function.identifier) {
            if entry.t != t {
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
                t,
                attrs: SymbolAttributes::Function {
                    defined: already_defined || has_body,
                    global,
                },
            },
        );

        let body = if let Some(body) = &declaration.body {
            for parameter in &declaration.parameters {
                self.symbols.insert(
                    parameter.identifier.clone(),
                    Symbol {
                        t: Type::Int,
                        attrs: SymbolAttributes::Local,
                    },
                );
            }

            Some(self.handle_block(body)?)
        } else {
            None
        };

        Ok(FunctionDeclaration {
            function: declaration.function.clone(),
            parameters: declaration.parameters.clone(),
            body,
            storage_class: declaration.storage_class,
        })
    }

    fn handle_block(&mut self, block: &Block) -> Result<Block, String> {
        let mut result = block.clone();

        for item in result.items.iter_mut() {
            match item {
                BlockItem::Statement(statement) => {
                    *item = BlockItem::Statement(self.handle_statement(statement)?);
                }
                BlockItem::Declaration(declaration) => {
                    *item =
                        BlockItem::Declaration(self.handle_block_level_declaration(declaration)?);
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
        let t = Type::Int;

        Ok(match declaration.storage_class {
            Some(StorageClass::Extern) => {
                if declaration.initializer.is_some() {
                    return Err(
                        "Block-level extern variable cannot have an initializer".to_string()
                    );
                }

                if let Some(entry) = self.symbols.get(&declaration.variable.identifier) {
                    if entry.t != t {
                        return Err(format!(
                            "Incompatible redeclaration of variable {}",
                            declaration.variable.identifier
                        ));
                    }
                } else {
                    self.symbols.insert(
                        declaration.variable.identifier.clone(),
                        Symbol {
                            t,
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
                let initial = match declaration.initializer {
                    Some(Expression::Constant(i)) => SymbolInitialValue::Initial(i),
                    None => SymbolInitialValue::Initial(0),
                    _ => {
                        return Err(
                            "Non-constant initializer on block-level static variable".to_string()
                        )
                    }
                };

                self.symbols.insert(
                    declaration.variable.identifier.clone(),
                    Symbol {
                        t,
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
                        t,
                        attrs: SymbolAttributes::Local,
                    },
                );

                let initializer = if let Some(expr) = &declaration.initializer {
                    Some(self.handle_expression(expr)?)
                } else {
                    None
                };

                VariableDeclaration {
                    variable: declaration.variable.clone(),
                    initializer,
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
            } => {
                let entry = self.symbols.get(&function.identifier).unwrap();

                let Type::Function { parameter_count } = entry.t else {
                    return Err(format!("{} is not a function", function.identifier));
                };

                if parameter_count != arguments.len() {
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

                let Type::Int = entry.t else {
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
