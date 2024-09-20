use crate::compiler::{
    ast::{BlockItem, Function, Label, Program, Statement},
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
        let mut block_items = program.function_definition.body.clone();

        for item in block_items.iter_mut() {
            if let BlockItem::Statement(statement) = item {
                *statement = self.rewrite_label_in_labeled_statement(statement)?;
            }
        }

        for item in block_items.iter_mut() {
            if let BlockItem::Statement(statement) = item {
                *statement = self.rewrite_goto_in_statement(statement)?;
            }
        }

        Ok(Program {
            function_definition: Function {
                name: program.function_definition.name.clone(),
                body: block_items,
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

    fn rewrite_label_in_labeled_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<Statement, String> {
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
                    Box::new(self.rewrite_label_in_labeled_statement(statement)?),
                )
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => Statement::If {
                condition: condition.clone(),
                then_branch: Box::new(self.rewrite_label_in_labeled_statement(then_branch)?),
                else_branch: if let Some(else_branch) = else_branch {
                    Some(Box::new(
                        self.rewrite_label_in_labeled_statement(else_branch)?,
                    ))
                } else {
                    None
                },
            },
            Statement::Return(_)
            | Statement::Expression(_)
            | Statement::Goto(_)
            | Statement::Null => statement.clone(),
        })
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
            Statement::Return(_) | Statement::Expression(_) | Statement::Null => statement.clone(),
        })
    }
}
