use std::collections::HashSet;

use crate::compiler::{
    ast::{
        Block, BlockItem, Constant, Declaration, Expression, Program, Statement, SwitchCaseLabel,
        SwitchCases,
    },
    constants::SEMANTIC_CASE_PREFIX,
};

pub struct SwitchCaseCollector {
    counter: usize,
}

impl SwitchCaseCollector {
    fn new() -> Self {
        Self { counter: 0 }
    }

    pub fn analyze(program: &Program) -> Result<Program, String> {
        let mut collector = Self::new();

        let mut result = program.clone();

        for declaration in result.declarations.iter_mut() {
            if let Declaration::Function(fd) = declaration {
                if let Some(body) = &fd.body {
                    let (new_body, cases) = collector.handle_block(body)?;

                    if cases.is_some() {
                        return Err(
                            "Unexpected switch case outside of switch statement".to_string()
                        );
                    }

                    fd.body = Some(new_body);
                }
            }
        }

        Ok(result)
    }

    fn fresh_switch_case_label(&mut self, suffix: Option<&str>) -> SwitchCaseLabel {
        let name = match suffix {
            Some(suffix) => format!("{SEMANTIC_CASE_PREFIX}.{}.{}", self.counter, suffix),
            None => format!("{SEMANTIC_CASE_PREFIX}.{}", self.counter),
        };
        self.counter += 1;

        SwitchCaseLabel { identifier: name }
    }

    fn merge_and_verify_switch_cases(
        lhs: &Option<SwitchCases>,
        rhs: &Option<SwitchCases>,
    ) -> Result<Option<SwitchCases>, String> {
        if lhs.is_none() || rhs.is_none() {
            return Ok(lhs.clone().or(rhs.clone()));
        }

        let lhs = lhs.clone().unwrap();
        let rhs = rhs.clone().unwrap();

        let mut merged = SwitchCases {
            cases: Vec::new(),
            default: None,
        };

        if lhs.default.is_some() && rhs.default.is_some() {
            return Err("Multiple default cases in switch statement".to_string());
        }

        merged.default = lhs.default.or(rhs.default);

        let mut set = HashSet::new();

        for (expr, case_label) in lhs.cases.iter().chain(rhs.cases.iter()) {
            let Expression::Constant { c, ty: _ } = expr else {
                return Err("Non-constant expression in switch case".to_string());
            };

            if set.contains(c) {
                return Err("Duplicate case value in switch statement".to_string());
            }

            set.insert(c.clone());
            merged.cases.push((expr.clone(), case_label.clone()));
        }

        Ok(Some(merged))
    }

    fn handle_block(&mut self, block: &Block) -> Result<(Block, Option<SwitchCases>), String> {
        let mut result = block.clone();

        let mut switch_cases = None;

        for item in result.items.iter_mut() {
            if let BlockItem::Statement(statement) = item {
                let (new_statement, new_switch_cases) = self.handle_statement(statement)?;

                *statement = new_statement;

                switch_cases =
                    Self::merge_and_verify_switch_cases(&switch_cases, &new_switch_cases)?;
            }
        }

        Ok((result, switch_cases))
    }

    fn handle_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<(Statement, Option<SwitchCases>), String> {
        Ok(match statement {
            Statement::Switch {
                expression,
                body,
                cases: _,
                label,
            } => {
                let (new_body, collected_cases) = self.handle_statement(body)?;

                (
                    Statement::Switch {
                        expression: expression.clone(),
                        body: Box::new(new_body),
                        cases: collected_cases,
                        label: label.clone(),
                    },
                    None,
                )
            }

            Statement::Case {
                expression,
                body,
                label: _,
            } => {
                let Expression::Constant { c, ty: _ } = expression else {
                    return Err("Non-constant expression in switch case".to_string());
                };

                let case_label = self.fresh_switch_case_label(Some(&format!(
                    "value.{}",
                    match c {
                        Constant::ConstantInt(n) => n.to_string(),
                        Constant::ConstantLong(n) => n.to_string(),
                    }
                )));
                let (new_body, inner_cases) = self.handle_statement(body)?;

                let merged = Self::merge_and_verify_switch_cases(
                    &Some(SwitchCases {
                        cases: vec![(expression.clone(), case_label.clone())],
                        default: None,
                    }),
                    &inner_cases,
                )?;

                (
                    Statement::Case {
                        expression: expression.clone(),
                        body: Box::new(new_body),
                        label: Some(case_label),
                    },
                    merged,
                )
            }
            Statement::Default { body, label: _ } => {
                let case_label = self.fresh_switch_case_label(Some("default"));
                let (new_body, inner_cases) = self.handle_statement(body)?;

                let merged = Self::merge_and_verify_switch_cases(
                    &Some(SwitchCases {
                        cases: Vec::new(),
                        default: Some(case_label.clone()),
                    }),
                    &inner_cases,
                )?;

                (
                    Statement::Default {
                        body: Box::new(new_body),
                        label: Some(case_label),
                    },
                    merged,
                )
            }

            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let (new_then_branch, then_cases) = self.handle_statement(then_branch)?;

                let mut merged = then_cases;

                let new_else_branch = if let Some(else_branch) = else_branch {
                    let (new_else_branch, new_else_cases) = self.handle_statement(else_branch)?;

                    merged = Self::merge_and_verify_switch_cases(&merged, &new_else_cases)?;

                    Some(Box::new(new_else_branch))
                } else {
                    None
                };

                (
                    Statement::If {
                        condition: condition.clone(),
                        then_branch: Box::new(new_then_branch),
                        else_branch: new_else_branch,
                    },
                    merged,
                )
            }
            Statement::Labeled(label, statement) => {
                let (new_statement, new_cases) = self.handle_statement(statement)?;

                (
                    Statement::Labeled(label.clone(), Box::new(new_statement)),
                    new_cases,
                )
            }
            Statement::Compound(block) => {
                let (new_block, new_cases) = self.handle_block(block)?;

                (Statement::Compound(new_block), new_cases)
            }
            Statement::While {
                condition,
                body,
                label,
            } => {
                let (new_body, new_cases) = self.handle_statement(body)?;

                (
                    Statement::While {
                        condition: condition.clone(),
                        body: Box::new(new_body),
                        label: label.clone(),
                    },
                    new_cases,
                )
            }
            Statement::DoWhile {
                body,
                condition,
                label,
            } => {
                let (new_body, new_cases) = self.handle_statement(body)?;

                (
                    Statement::DoWhile {
                        body: Box::new(new_body),
                        condition: condition.clone(),
                        label: label.clone(),
                    },
                    new_cases,
                )
            }
            Statement::For {
                initializer,
                condition,
                post,
                body,
                label,
            } => {
                let (new_body, new_cases) = self.handle_statement(body)?;

                (
                    Statement::For {
                        initializer: initializer.clone(),
                        condition: condition.clone(),
                        post: post.clone(),
                        body: Box::new(new_body),
                        label: label.clone(),
                    },
                    new_cases,
                )
            }

            Statement::Null
            | Statement::Return(_)
            | Statement::Expression(_)
            | Statement::Goto(_)
            | Statement::Break(_)
            | Statement::Continue(_) => (statement.clone(), None),
        })
    }
}
