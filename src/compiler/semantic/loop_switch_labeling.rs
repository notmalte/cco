use crate::compiler::{
    ast::{
        Block, BlockItem, Declaration, LoopLabel, LoopOrSwitchLabel, Program, Statement,
        SwitchLabel,
    },
    prefixes::{SEMANTIC_LOOP_PREFIX, SEMANTIC_SWITCH_PREFIX},
};

struct Enclosing {
    breakable: Option<LoopOrSwitchLabel>,
    continuable: Option<LoopLabel>,
}

pub struct LoopSwitchLabeler {
    loop_counter: usize,
    switch_counter: usize,
}

impl LoopSwitchLabeler {
    fn new() -> Self {
        Self {
            loop_counter: 0,
            switch_counter: 0,
        }
    }

    pub fn analyze(program: &Program) -> Result<Program, String> {
        let mut labeler = Self::new();

        let mut result = program.clone();

        for declaration in result.declarations.iter_mut() {
            if let Declaration::Function(fd) = declaration {
                if let Some(body) = &fd.body {
                    fd.body = Some(labeler.handle_block(
                        body,
                        &Enclosing {
                            breakable: None,
                            continuable: None,
                        },
                    )?);
                }
            }
        }

        Ok(result)
    }

    fn fresh_loop_label(&mut self, suffix: Option<&str>) -> LoopLabel {
        let name = match suffix {
            Some(suffix) => format!("{SEMANTIC_LOOP_PREFIX}.{}.{}", self.loop_counter, suffix),
            None => format!("{SEMANTIC_LOOP_PREFIX}.{}", self.loop_counter),
        };
        self.loop_counter += 1;

        LoopLabel { identifier: name }
    }

    fn fresh_switch_label(&mut self) -> SwitchLabel {
        let name = format!("{SEMANTIC_SWITCH_PREFIX}.{}", self.switch_counter);
        self.switch_counter += 1;

        SwitchLabel { identifier: name }
    }

    fn handle_block(&mut self, block: &Block, enclosing: &Enclosing) -> Result<Block, String> {
        let mut result = block.clone();
        for item in result.items.iter_mut() {
            if let BlockItem::Statement(statement) = item {
                *statement = self.handle_statement(statement, enclosing)?;
            }
        }
        Ok(result)
    }

    fn handle_statement(
        &mut self,
        statement: &Statement,
        enclosing: &Enclosing,
    ) -> Result<Statement, String> {
        Ok(match statement {
            Statement::Break(_) => Statement::Break(Some(
                enclosing
                    .breakable
                    .clone()
                    .ok_or("Break statement outside of loop or switch".to_string())?,
            )),
            Statement::Continue(_) => Statement::Continue(Some(
                enclosing
                    .continuable
                    .clone()
                    .ok_or("Continue statement outside of loop".to_string())?,
            )),

            Statement::While {
                condition,
                body,
                label: _,
            } => {
                let fresh = self.fresh_loop_label(Some("while"));

                Statement::While {
                    condition: condition.clone(),
                    body: Box::new(self.handle_statement(
                        body,
                        &Enclosing {
                            breakable: Some(LoopOrSwitchLabel::Loop(fresh.clone())),
                            continuable: Some(fresh.clone()),
                        },
                    )?),
                    label: Some(fresh),
                }
            }
            Statement::DoWhile {
                body,
                condition,
                label: _,
            } => {
                let fresh = self.fresh_loop_label(Some("do"));

                Statement::DoWhile {
                    body: Box::new(self.handle_statement(
                        body,
                        &Enclosing {
                            breakable: Some(LoopOrSwitchLabel::Loop(fresh.clone())),
                            continuable: Some(fresh.clone()),
                        },
                    )?),
                    condition: condition.clone(),
                    label: Some(fresh),
                }
            }
            Statement::For {
                initializer,
                condition,
                post,
                body,
                label: _,
            } => {
                let fresh = self.fresh_loop_label(Some("for"));

                Statement::For {
                    initializer: initializer.clone(),
                    condition: condition.clone(),
                    post: post.clone(),
                    body: Box::new(self.handle_statement(
                        body,
                        &Enclosing {
                            breakable: Some(LoopOrSwitchLabel::Loop(fresh.clone())),
                            continuable: Some(fresh.clone()),
                        },
                    )?),
                    label: Some(fresh),
                }
            }

            Statement::Switch {
                expression,
                body,
                cases,
                label: _,
            } => {
                let fresh = self.fresh_switch_label();

                Statement::Switch {
                    expression: expression.clone(),
                    body: Box::new(self.handle_statement(
                        body,
                        &Enclosing {
                            breakable: Some(LoopOrSwitchLabel::Switch(fresh.clone())),
                            continuable: enclosing.continuable.clone(),
                        },
                    )?),
                    cases: cases.clone(),
                    label: Some(fresh),
                }
            }

            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => Statement::If {
                condition: condition.clone(),
                then_branch: Box::new(self.handle_statement(then_branch, enclosing)?),
                else_branch: if let Some(else_branch) = else_branch {
                    Some(Box::new(self.handle_statement(else_branch, enclosing)?))
                } else {
                    None
                },
            },
            Statement::Labeled(label, statement) => Statement::Labeled(
                label.clone(),
                Box::new(self.handle_statement(statement, enclosing)?),
            ),
            Statement::Compound(block) => Statement::Compound(self.handle_block(block, enclosing)?),
            Statement::Case {
                expression,
                body,
                label,
            } => Statement::Case {
                expression: expression.clone(),
                body: Box::new(self.handle_statement(body, enclosing)?),
                label: label.clone(),
            },
            Statement::Default { body, label } => Statement::Default {
                body: Box::new(self.handle_statement(body, enclosing)?),
                label: label.clone(),
            },

            Statement::Null
            | Statement::Return(_)
            | Statement::Expression(_)
            | Statement::Goto(_) => statement.clone(),
        })
    }
}
