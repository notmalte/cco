use crate::compiler::{
    ast::{Block, BlockItem, Function, Label, Program, Statement},
    constants::SEMANTIC_LABEL_PREFIX,
};
use std::collections::HashMap;

pub struct LabelResolver {
    map: HashMap<String, String>,
    counter: usize,
}

impl LabelResolver {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            counter: 0,
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<Program, String> {
        let mut body = program.function_definition.body.clone();

        body = self.rewrite_label_in_block(&body)?;

        body = self.rewrite_goto_in_block(&body)?;

        Ok(Program {
            function_definition: Function {
                name: program.function_definition.name.clone(),
                body,
            },
        })
    }

    fn fresh_label(&mut self, suffix: Option<&str>) -> Label {
        let name = match suffix {
            Some(suffix) => format!("{SEMANTIC_LABEL_PREFIX}.{}.{}", self.counter, suffix),
            None => format!("{SEMANTIC_LABEL_PREFIX}.{}", self.counter),
        };
        self.counter += 1;

        Label { identifier: name }
    }

    fn rewrite_label_in_block(&mut self, block: &Block) -> Result<Block, String> {
        let mut result = block.clone();
        for item in result.items.iter_mut() {
            if let BlockItem::Statement(statement) = item {
                *statement = self.rewrite_label_in_statement(statement)?;
            }
        }
        Ok(result)
    }

    fn rewrite_label_in_statement(&mut self, statement: &Statement) -> Result<Statement, String> {
        Ok(match statement {
            Statement::Labeled(label, statement) => {
                if self.map.contains_key(&label.identifier) {
                    return Err(format!("Label {} already declared", label.identifier));
                }

                let new_label = self.fresh_label(Some(&label.identifier));
                self.map
                    .insert(label.identifier.clone(), new_label.identifier.clone());

                Statement::Labeled(
                    new_label,
                    Box::new(self.rewrite_label_in_statement(statement)?),
                )
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => Statement::If {
                condition: condition.clone(),
                then_branch: Box::new(self.rewrite_label_in_statement(then_branch)?),
                else_branch: if let Some(else_branch) = else_branch {
                    Some(Box::new(self.rewrite_label_in_statement(else_branch)?))
                } else {
                    None
                },
            },
            Statement::Compound(block) => Statement::Compound(self.rewrite_label_in_block(block)?),
            Statement::Return(_)
            | Statement::Expression(_)
            | Statement::Goto(_)
            | Statement::Null => statement.clone(),
        })
    }

    fn rewrite_goto_in_block(&mut self, block: &Block) -> Result<Block, String> {
        let mut result = block.clone();
        for item in result.items.iter_mut() {
            if let BlockItem::Statement(statement) = item {
                *statement = self.rewrite_goto_in_statement(statement)?;
            }
        }
        Ok(result)
    }

    fn rewrite_goto_in_statement(&mut self, statement: &Statement) -> Result<Statement, String> {
        Ok(match statement {
            Statement::Goto(label) => {
                if let Some(new_name) = self.map.get(&label.identifier) {
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
                then_branch: Box::new(self.rewrite_goto_in_statement(then_branch)?),
                else_branch: if let Some(else_branch) = else_branch {
                    Some(Box::new(self.rewrite_goto_in_statement(else_branch)?))
                } else {
                    None
                },
            },
            Statement::Labeled(label, statement) => Statement::Labeled(
                label.clone(),
                Box::new(self.rewrite_goto_in_statement(statement)?),
            ),
            Statement::Compound(block) => Statement::Compound(self.rewrite_goto_in_block(block)?),
            Statement::Return(_) | Statement::Expression(_) | Statement::Null => statement.clone(),
        })
    }
}
