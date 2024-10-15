use crate::compiler::{
    ast::{Block, BlockItem, Declaration, FunctionDeclaration, Label, Program, Statement},
    constants::SEMANTIC_LABEL_PREFIX,
};
use std::collections::HashMap;

type LabelMap = HashMap<String, String>;

pub struct LabelResolver {
    counter: usize,
}

impl LabelResolver {
    fn new() -> Self {
        Self { counter: 0 }
    }

    pub fn analyze(program: &Program) -> Result<Program, String> {
        let mut resolver = Self::new();

        let mut result = program.clone();

        for declaration in result.declarations.iter_mut() {
            if let Declaration::Function(fd) = declaration {
                *fd = resolver.handle_function_declaration(fd)?;
            }
        }

        Ok(result)
    }

    fn fresh_label(&mut self, suffix: Option<&str>) -> Label {
        let name = match suffix {
            Some(suffix) => format!("{SEMANTIC_LABEL_PREFIX}.{}.{}", self.counter, suffix),
            None => format!("{SEMANTIC_LABEL_PREFIX}.{}", self.counter),
        };
        self.counter += 1;

        Label { identifier: name }
    }

    fn handle_function_declaration(
        &mut self,
        fd: &FunctionDeclaration,
    ) -> Result<FunctionDeclaration, String> {
        if let Some(body) = &fd.body {
            let mut map = LabelMap::new();

            let mut body = body.clone();
            body = self.rewrite_label_in_block(&body, &mut map)?;
            body = self.rewrite_goto_in_block(&body, &mut map)?;

            Ok(FunctionDeclaration {
                function: fd.function.clone(),
                parameters: fd.parameters.clone(),
                body: Some(body),
                storage_class: fd.storage_class,
            })
        } else {
            Ok(fd.clone())
        }
    }

    fn rewrite_label_in_block(
        &mut self,
        block: &Block,
        map: &mut LabelMap,
    ) -> Result<Block, String> {
        let mut result = block.clone();
        for item in result.items.iter_mut() {
            if let BlockItem::Statement(statement) = item {
                *statement = self.rewrite_label_in_statement(statement, map)?;
            }
        }
        Ok(result)
    }

    fn rewrite_label_in_statement(
        &mut self,
        statement: &Statement,
        map: &mut LabelMap,
    ) -> Result<Statement, String> {
        Ok(match statement {
            Statement::Labeled(label, statement) => {
                if map.contains_key(&label.identifier) {
                    return Err(format!("Label {} already declared", label.identifier));
                }

                let new_label = self.fresh_label(Some(&label.identifier));
                map.insert(label.identifier.clone(), new_label.identifier.clone());

                Statement::Labeled(
                    new_label,
                    Box::new(self.rewrite_label_in_statement(statement, map)?),
                )
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => Statement::If {
                condition: condition.clone(),
                then_branch: Box::new(self.rewrite_label_in_statement(then_branch, map)?),
                else_branch: if let Some(else_branch) = else_branch {
                    Some(Box::new(self.rewrite_label_in_statement(else_branch, map)?))
                } else {
                    None
                },
            },
            Statement::Compound(block) => {
                Statement::Compound(self.rewrite_label_in_block(block, map)?)
            }
            Statement::While {
                condition,
                body,
                label,
            } => Statement::While {
                condition: condition.clone(),
                body: Box::new(self.rewrite_label_in_statement(body, map)?),
                label: label.clone(),
            },
            Statement::DoWhile {
                body,
                condition,
                label,
            } => Statement::DoWhile {
                body: Box::new(self.rewrite_label_in_statement(body, map)?),
                condition: condition.clone(),
                label: label.clone(),
            },
            Statement::For {
                initializer,
                condition,
                post,
                body,
                label,
            } => Statement::For {
                initializer: initializer.clone(),
                condition: condition.clone(),
                post: post.clone(),
                body: Box::new(self.rewrite_label_in_statement(body, map)?),
                label: label.clone(),
            },

            Statement::Null
            | Statement::Return(_)
            | Statement::Expression(_)
            | Statement::Goto(_)
            | Statement::Break(_)
            | Statement::Continue(_) => statement.clone(),
            Statement::Switch { .. } | Statement::Case { .. } | Statement::Default { .. } => {
                todo!()
            }
        })
    }

    fn rewrite_goto_in_block(
        &mut self,
        block: &Block,
        map: &mut LabelMap,
    ) -> Result<Block, String> {
        let mut result = block.clone();
        for item in result.items.iter_mut() {
            if let BlockItem::Statement(statement) = item {
                *statement = self.rewrite_goto_in_statement(statement, map)?;
            }
        }
        Ok(result)
    }

    fn rewrite_goto_in_statement(
        &mut self,
        statement: &Statement,
        map: &mut LabelMap,
    ) -> Result<Statement, String> {
        Ok(match statement {
            Statement::Goto(label) => {
                if let Some(new_name) = map.get(&label.identifier) {
                    Statement::Goto(Label {
                        identifier: new_name.clone(),
                    })
                } else {
                    return Err(format!("Label {} not declared", label.identifier));
                }
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => Statement::If {
                condition: condition.clone(),
                then_branch: Box::new(self.rewrite_goto_in_statement(then_branch, map)?),
                else_branch: if let Some(else_branch) = else_branch {
                    Some(Box::new(self.rewrite_goto_in_statement(else_branch, map)?))
                } else {
                    None
                },
            },
            Statement::Labeled(label, statement) => Statement::Labeled(
                label.clone(),
                Box::new(self.rewrite_goto_in_statement(statement, map)?),
            ),
            Statement::Compound(block) => {
                Statement::Compound(self.rewrite_goto_in_block(block, map)?)
            }
            Statement::While {
                condition,
                body,
                label,
            } => Statement::While {
                condition: condition.clone(),
                body: Box::new(self.rewrite_goto_in_statement(body, map)?),
                label: label.clone(),
            },
            Statement::DoWhile {
                body,
                condition,
                label,
            } => Statement::DoWhile {
                body: Box::new(self.rewrite_goto_in_statement(body, map)?),
                condition: condition.clone(),
                label: label.clone(),
            },
            Statement::For {
                initializer,
                condition,
                post,
                body,
                label,
            } => Statement::For {
                initializer: initializer.clone(),
                condition: condition.clone(),
                post: post.clone(),
                body: Box::new(self.rewrite_goto_in_statement(body, map)?),
                label: label.clone(),
            },

            Statement::Null
            | Statement::Return(_)
            | Statement::Expression(_)
            | Statement::Break(_)
            | Statement::Continue(_) => statement.clone(),
            Statement::Switch { .. } | Statement::Case { .. } | Statement::Default { .. } => {
                todo!()
            }
        })
    }
}
