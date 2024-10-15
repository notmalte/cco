use crate::compiler::{
    ast::{Block, BlockItem, Declaration, LoopLabel, Program, Statement},
    constants::SEMANTIC_LOOP_PREFIX,
};

pub struct LoopLabeler {
    counter: usize,
}

impl LoopLabeler {
    fn new() -> Self {
        Self { counter: 0 }
    }

    pub fn analyze(program: &Program) -> Result<Program, String> {
        let mut labeler = Self::new();

        let mut result = program.clone();

        for declaration in result.declarations.iter_mut() {
            if let Declaration::Function(fd) = declaration {
                if let Some(body) = &fd.body {
                    fd.body = Some(labeler.handle_block(body, None)?);
                }
            }
        }

        Ok(result)
    }

    fn fresh_loop_label(&mut self, suffix: Option<&str>) -> LoopLabel {
        let name = match suffix {
            Some(suffix) => format!("{SEMANTIC_LOOP_PREFIX}.{}.{}", self.counter, suffix),
            None => format!("{SEMANTIC_LOOP_PREFIX}.{}", self.counter),
        };
        self.counter += 1;

        LoopLabel { identifier: name }
    }

    fn handle_block(
        &mut self,
        block: &Block,
        enclosing: Option<&LoopLabel>,
    ) -> Result<Block, String> {
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
        enclosing: Option<&LoopLabel>,
    ) -> Result<Statement, String> {
        Ok(match statement {
            Statement::Break(_) => Statement::Break(Some(
                enclosing
                    .ok_or("Break statement outside of loop".to_string())?
                    .clone(),
            )),
            Statement::Continue(_) => Statement::Continue(Some(
                enclosing
                    .ok_or("Continue statement outside of loop".to_string())?
                    .clone(),
            )),

            Statement::While {
                condition,
                body,
                label: _,
            } => {
                let fresh = self.fresh_loop_label(Some("while"));

                Statement::While {
                    condition: condition.clone(),
                    body: Box::new(self.handle_statement(body, Some(&fresh))?),
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
                    body: Box::new(self.handle_statement(body, Some(&fresh))?),
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
                    body: Box::new(self.handle_statement(body, Some(&fresh))?),
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

            Statement::Null
            | Statement::Return(_)
            | Statement::Expression(_)
            | Statement::Goto(_) => statement.clone(),
            Statement::Switch { .. } | Statement::Case { .. } | Statement::Default { .. } => {
                todo!()
            }
        })
    }
}
