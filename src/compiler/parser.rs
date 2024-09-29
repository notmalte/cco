use std::collections::VecDeque;

use super::{
    ast::{
        AssignmentOperator, BinaryOperator, Block, BlockItem, Declaration, Expression,
        ForInitializer, Function, FunctionDeclaration, Label, Program, Statement, UnaryOperator,
        Variable, VariableDeclaration,
    },
    token::Token,
};

pub fn parse(tokens: &[Token]) -> Result<Program, String> {
    let mut tokens = VecDeque::from_iter(tokens.iter().cloned());

    let program = parse_program(&mut tokens)?;

    if !tokens.is_empty() {
        return Err("Expected EOF".to_string());
    }

    Ok(program)
}

fn parse_program(tokens: &mut VecDeque<Token>) -> Result<Program, String> {
    Ok(Program {
        functions: parse_function_declarations(tokens)?,
    })
}

fn parse_function_declarations(
    tokens: &mut VecDeque<Token>,
) -> Result<Vec<FunctionDeclaration>, String> {
    let mut functions = vec![];

    while !tokens.is_empty() {
        functions.push(parse_function_declaration(tokens)?);
    }

    Ok(functions)
}

fn parse_function_declaration(tokens: &mut VecDeque<Token>) -> Result<FunctionDeclaration, String> {
    let Some(Token::IntKeyword) = tokens.pop_front() else {
        return Err("Expected int keyword".to_string());
    };

    let Some(Token::Identifier(identifier)) = tokens.pop_front() else {
        return Err("Expected identifier".to_string());
    };

    let Some(Token::OpenParen) = tokens.pop_front() else {
        return Err("Expected open parenthesis".to_string());
    };

    let parameters = parse_parameters(tokens)?;

    let Some(Token::CloseParen) = tokens.pop_front() else {
        return Err("Expected close parenthesis".to_string());
    };

    let body = if let Some(Token::Semicolon) = tokens.front() {
        tokens.pop_front();
        None
    } else {
        Some(parse_block(tokens)?)
    };

    Ok(FunctionDeclaration {
        function: Function { identifier },
        parameters,
        body,
    })
}

fn parse_parameters(tokens: &mut VecDeque<Token>) -> Result<Vec<Variable>, String> {
    if let Some(Token::VoidKeyword) = tokens.front() {
        tokens.pop_front();
        return Ok(vec![]);
    }

    let mut parameters = vec![];

    loop {
        let Some(Token::IntKeyword) = tokens.pop_front() else {
            return Err("Expected int keyword".to_string());
        };

        let Some(Token::Identifier(identifier)) = tokens.pop_front() else {
            return Err("Expected identifier".to_string());
        };

        parameters.push(Variable { identifier });

        if let Some(Token::Comma) = tokens.front() {
            tokens.pop_front();
        } else {
            break;
        }
    }

    Ok(parameters)
}

fn parse_block(tokens: &mut VecDeque<Token>) -> Result<Block, String> {
    let Some(Token::OpenBrace) = tokens.pop_front() else {
        return Err("Expected open brace".to_string());
    };

    let mut items = vec![];

    while let Some(t) = tokens.front() {
        if t == &Token::CloseBrace {
            break;
        }

        items.push(parse_block_item(tokens)?);
    }

    let Some(Token::CloseBrace) = tokens.pop_front() else {
        return Err("Expected close brace".to_string());
    };

    Ok(Block { items })
}

fn parse_block_item(tokens: &mut VecDeque<Token>) -> Result<BlockItem, String> {
    if let Some(Token::IntKeyword) = tokens.front() {
        parse_declaration(tokens).map(BlockItem::Declaration)
    } else {
        parse_statement(tokens).map(BlockItem::Statement)
    }
}

fn parse_declaration(tokens: &mut VecDeque<Token>) -> Result<Declaration, String> {
    let Some(Token::IntKeyword) = tokens.pop_front() else {
        return Err("Expected int keyword".to_string());
    };

    let Some(Token::Identifier(identifier)) = tokens.pop_front() else {
        return Err("Expected identifier".to_string());
    };

    if let Some(Token::OpenParen) = tokens.front() {
        tokens.pop_front();
        let parameters = parse_parameters(tokens)?;

        let Some(Token::CloseParen) = tokens.pop_front() else {
            return Err("Expected close parenthesis".to_string());
        };

        let body = if let Some(Token::Semicolon) = tokens.front() {
            tokens.pop_front();
            None
        } else {
            Some(parse_block(tokens)?)
        };

        Ok(Declaration::Function(FunctionDeclaration {
            function: Function { identifier },
            parameters,
            body,
        }))
    } else {
        let initializer = if let Some(Token::Equal) = tokens.front() {
            tokens.pop_front();
            let expression = parse_expression(tokens, 0)?;

            Some(expression)
        } else {
            None
        };

        let Some(Token::Semicolon) = tokens.pop_front() else {
            return Err("Expected semicolon".to_string());
        };

        Ok(Declaration::Variable(VariableDeclaration {
            variable: Variable { identifier },
            initializer,
        }))
    }
}

fn parse_statement(tokens: &mut VecDeque<Token>) -> Result<Statement, String> {
    match tokens.front() {
        Some(Token::Semicolon) => parse_null_statement(tokens),
        Some(Token::ReturnKeyword) => parse_return_statement(tokens),
        Some(Token::IfKeyword) => parse_if_statement(tokens),
        Some(Token::OpenBrace) => parse_block_statement(tokens),
        Some(Token::GotoKeyword) => parse_goto_statement(tokens),
        Some(Token::BreakKeyword) => parse_break_statement(tokens),
        Some(Token::ContinueKeyword) => parse_continue_statement(tokens),
        Some(Token::WhileKeyword) => parse_while_statement(tokens),
        Some(Token::DoKeyword) => parse_do_while_statement(tokens),
        Some(Token::ForKeyword) => parse_for_statement(tokens),
        Some(Token::Identifier(_)) => {
            if let Some(Token::Colon) = tokens.get(1) {
                parse_labeled_statement(tokens)
            } else {
                parse_expression_statement(tokens)
            }
        }
        _ => parse_expression_statement(tokens),
    }
}

fn parse_null_statement(tokens: &mut VecDeque<Token>) -> Result<Statement, String> {
    let Some(Token::Semicolon) = tokens.pop_front() else {
        return Err("Expected semicolon".to_string());
    };

    Ok(Statement::Null)
}

fn parse_return_statement(tokens: &mut VecDeque<Token>) -> Result<Statement, String> {
    let Some(Token::ReturnKeyword) = tokens.pop_front() else {
        return Err("Expected return keyword".to_string());
    };

    let expression = parse_expression(tokens, 0)?;

    let Some(Token::Semicolon) = tokens.pop_front() else {
        return Err("Expected semicolon".to_string());
    };

    Ok(Statement::Return(expression))
}

fn parse_if_statement(tokens: &mut VecDeque<Token>) -> Result<Statement, String> {
    let Some(Token::IfKeyword) = tokens.pop_front() else {
        return Err("Expected if keyword".to_string());
    };

    let Some(Token::OpenParen) = tokens.pop_front() else {
        return Err("Expected open parenthesis".to_string());
    };

    let condition = parse_expression(tokens, 0)?;

    let Some(Token::CloseParen) = tokens.pop_front() else {
        return Err("Expected close parenthesis".to_string());
    };

    let then_branch = Box::new(parse_statement(tokens)?);

    let else_branch = if let Some(Token::ElseKeyword) = tokens.front() {
        tokens.pop_front();
        Some(Box::new(parse_statement(tokens)?))
    } else {
        None
    };

    Ok(Statement::If {
        condition,
        then_branch,
        else_branch,
    })
}

fn parse_block_statement(tokens: &mut VecDeque<Token>) -> Result<Statement, String> {
    Ok(Statement::Compound(parse_block(tokens)?))
}

fn parse_goto_statement(tokens: &mut VecDeque<Token>) -> Result<Statement, String> {
    let Some(Token::GotoKeyword) = tokens.pop_front() else {
        return Err("Expected goto keyword".to_string());
    };

    let Some(Token::Identifier(label)) = tokens.pop_front() else {
        return Err("Expected identifier".to_string());
    };

    let Some(Token::Semicolon) = tokens.pop_front() else {
        return Err("Expected semicolon".to_string());
    };

    Ok(Statement::Goto(Label { identifier: label }))
}

fn parse_break_statement(tokens: &mut VecDeque<Token>) -> Result<Statement, String> {
    let Some(Token::BreakKeyword) = tokens.pop_front() else {
        return Err("Expected break keyword".to_string());
    };

    let Some(Token::Semicolon) = tokens.pop_front() else {
        return Err("Expected semicolon".to_string());
    };

    Ok(Statement::Break(None))
}

fn parse_continue_statement(tokens: &mut VecDeque<Token>) -> Result<Statement, String> {
    let Some(Token::ContinueKeyword) = tokens.pop_front() else {
        return Err("Expected continue keyword".to_string());
    };

    let Some(Token::Semicolon) = tokens.pop_front() else {
        return Err("Expected semicolon".to_string());
    };

    Ok(Statement::Continue(None))
}

fn parse_while_statement(tokens: &mut VecDeque<Token>) -> Result<Statement, String> {
    let Some(Token::WhileKeyword) = tokens.pop_front() else {
        return Err("Expected while keyword".to_string());
    };

    let Some(Token::OpenParen) = tokens.pop_front() else {
        return Err("Expected open parenthesis".to_string());
    };

    let condition = parse_expression(tokens, 0)?;

    let Some(Token::CloseParen) = tokens.pop_front() else {
        return Err("Expected close parenthesis".to_string());
    };

    let body = Box::new(parse_statement(tokens)?);

    Ok(Statement::While {
        condition,
        body,
        label: None,
    })
}

fn parse_do_while_statement(tokens: &mut VecDeque<Token>) -> Result<Statement, String> {
    let Some(Token::DoKeyword) = tokens.pop_front() else {
        return Err("Expected do keyword".to_string());
    };

    let body = Box::new(parse_statement(tokens)?);

    let Some(Token::WhileKeyword) = tokens.pop_front() else {
        return Err("Expected while keyword".to_string());
    };

    let Some(Token::OpenParen) = tokens.pop_front() else {
        return Err("Expected open parenthesis".to_string());
    };

    let condition = parse_expression(tokens, 0)?;

    let Some(Token::CloseParen) = tokens.pop_front() else {
        return Err("Expected close parenthesis".to_string());
    };

    let Some(Token::Semicolon) = tokens.pop_front() else {
        return Err("Expected semicolon".to_string());
    };

    Ok(Statement::DoWhile {
        body,
        condition,
        label: None,
    })
}

fn parse_for_statement(tokens: &mut VecDeque<Token>) -> Result<Statement, String> {
    let Some(Token::ForKeyword) = tokens.pop_front() else {
        return Err("Expected for keyword".to_string());
    };

    let Some(Token::OpenParen) = tokens.pop_front() else {
        return Err("Expected open parenthesis".to_string());
    };

    let initializer = parse_for_initializer(tokens)?;

    let condition = if let Some(Token::Semicolon) = tokens.front() {
        None
    } else {
        Some(parse_expression(tokens, 0)?)
    };

    let Some(Token::Semicolon) = tokens.pop_front() else {
        return Err("Expected semicolon".to_string());
    };

    let post = if let Some(Token::CloseParen) = tokens.front() {
        None
    } else {
        Some(parse_expression(tokens, 0)?)
    };

    let Some(Token::CloseParen) = tokens.pop_front() else {
        return Err("Expected close parenthesis".to_string());
    };

    let body = Box::new(parse_statement(tokens)?);

    Ok(Statement::For {
        initializer,
        condition,
        post,
        body,
        label: None,
    })
}

fn parse_for_initializer(tokens: &mut VecDeque<Token>) -> Result<Option<ForInitializer>, String> {
    if let Some(Token::Semicolon) = tokens.front() {
        tokens.pop_front();
        return Ok(None);
    }

    if let Some(Token::IntKeyword) = tokens.front() {
        let declaration = parse_declaration(tokens)?;

        let Declaration::Variable(vd) = declaration else {
            return Err("Expected variable declaration".to_string());
        };

        Ok(Some(ForInitializer::VariableDeclaration(vd)))
    } else {
        let expression = parse_expression(tokens, 0)?;
        tokens.pop_front();
        Ok(Some(ForInitializer::Expression(expression)))
    }
}

fn parse_labeled_statement(tokens: &mut VecDeque<Token>) -> Result<Statement, String> {
    let Some(Token::Identifier(label)) = tokens.pop_front() else {
        return Err("Expected identifier".to_string());
    };

    let Some(Token::Colon) = tokens.pop_front() else {
        return Err("Expected colon".to_string());
    };

    let statement = parse_statement(tokens)?;

    Ok(Statement::Labeled(
        Label { identifier: label },
        Box::new(statement),
    ))
}

fn parse_expression_statement(tokens: &mut VecDeque<Token>) -> Result<Statement, String> {
    let expression = parse_expression(tokens, 0)?;

    let Some(Token::Semicolon) = tokens.pop_front() else {
        return Err("Expected semicolon".to_string());
    };

    Ok(Statement::Expression(expression))
}

fn parse_expression(
    tokens: &mut VecDeque<Token>,
    min_precedence: u8,
) -> Result<Expression, String> {
    let mut left = parse_factor(tokens)?;
    while let Some(t) = tokens.front() {
        let precedence = match t {
            Token::Equal
            | Token::PlusEqual
            | Token::MinusEqual
            | Token::AsteriskEqual
            | Token::SlashEqual
            | Token::PercentEqual
            | Token::AmpersandEqual
            | Token::PipeEqual
            | Token::CaretEqual
            | Token::LessLessEqual
            | Token::GreaterGreaterEqual => 1,
            Token::Question => 2,
            Token::PipePipe => 3,
            Token::AmpersandAmpersand => 4,
            Token::Pipe => 5,
            Token::Caret => 6,
            Token::Ampersand => 7,
            Token::EqualEqual | Token::ExclamationEqual => 8,
            Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual => 9,
            Token::LessLess | Token::GreaterGreater => 10,
            Token::Plus | Token::Minus => 11,
            Token::Asterisk | Token::Slash | Token::Percent => 12,
            _ => break,
        };

        if precedence < min_precedence {
            break;
        }

        match t {
            Token::Equal
            | Token::PlusEqual
            | Token::MinusEqual
            | Token::AsteriskEqual
            | Token::SlashEqual
            | Token::PercentEqual
            | Token::AmpersandEqual
            | Token::PipeEqual
            | Token::CaretEqual
            | Token::LessLessEqual
            | Token::GreaterGreaterEqual => {
                let op = parse_assignment_operator(tokens)?;
                let right = parse_expression(tokens, precedence)?;
                left = Expression::Assignment {
                    op,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                };
            }
            Token::Question => {
                let Some(Token::Question) = tokens.pop_front() else {
                    return Err("Expected question mark".to_string());
                };

                let then_expr = parse_expression(tokens, 0)?;

                let Some(Token::Colon) = tokens.pop_front() else {
                    return Err("Expected colon".to_string());
                };

                let else_expr = parse_expression(tokens, precedence)?;

                left = Expression::Conditional {
                    condition: Box::new(left),
                    then_expr: Box::new(then_expr),
                    else_expr: Box::new(else_expr),
                };
            }
            _ => {
                let op = parse_binary_operator(tokens)?;
                let right = parse_expression(tokens, precedence + 1)?;
                left = Expression::Binary {
                    op,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                };
            }
        }
    }
    Ok(left)
}

fn parse_factor(tokens: &mut VecDeque<Token>) -> Result<Expression, String> {
    let mut factor = match tokens.front().cloned() {
        Some(Token::Constant(value)) => {
            tokens.pop_front();
            Expression::Constant(value)
        }
        Some(Token::Identifier(identifier)) => {
            tokens.pop_front();

            if let Some(Token::OpenParen) = tokens.front() {
                tokens.pop_front();

                let mut arguments = vec![];

                if tokens.front() != Some(&Token::CloseParen) {
                    loop {
                        arguments.push(parse_expression(tokens, 0)?);

                        if let Some(Token::Comma) = tokens.front() {
                            tokens.pop_front();
                        } else {
                            break;
                        }
                    }
                }

                let Some(Token::CloseParen) = tokens.pop_front() else {
                    return Err("Expected close parenthesis".to_string());
                };

                Expression::FunctionCall {
                    function: Function { identifier },
                    arguments,
                }
            } else {
                Expression::Variable(Variable { identifier })
            }
        }
        Some(
            Token::Tilde | Token::Minus | Token::Exclamation | Token::PlusPlus | Token::MinusMinus,
        ) => {
            let op = parse_unary_prefix_operator(tokens)?;
            let inner = parse_factor(tokens)?;
            Expression::Unary {
                op,
                expr: Box::new(inner),
            }
        }
        Some(Token::OpenParen) => {
            tokens.pop_front();
            let inner = parse_expression(tokens, 0)?;
            let Some(Token::CloseParen) = tokens.pop_front() else {
                return Err("Expected close parenthesis".to_string());
            };
            inner
        }
        _ => return Err("Expected factor".to_string()),
    };

    while let Some(Token::PlusPlus | Token::MinusMinus) = tokens.front() {
        let op = parse_unary_postfix_operator(tokens)?;
        factor = Expression::Unary {
            op,
            expr: Box::new(factor),
        };
    }

    Ok(factor)
}

fn parse_unary_prefix_operator(tokens: &mut VecDeque<Token>) -> Result<UnaryOperator, String> {
    match tokens.pop_front() {
        Some(Token::Tilde) => Ok(UnaryOperator::Complement),
        Some(Token::Minus) => Ok(UnaryOperator::Negate),
        Some(Token::Exclamation) => Ok(UnaryOperator::Not),
        Some(Token::PlusPlus) => Ok(UnaryOperator::PrefixIncrement),
        Some(Token::MinusMinus) => Ok(UnaryOperator::PrefixDecrement),
        _ => Err("Expected unary prefix operator".to_string()),
    }
}

fn parse_unary_postfix_operator(tokens: &mut VecDeque<Token>) -> Result<UnaryOperator, String> {
    match tokens.pop_front() {
        Some(Token::PlusPlus) => Ok(UnaryOperator::PostfixIncrement),
        Some(Token::MinusMinus) => Ok(UnaryOperator::PostfixDecrement),
        _ => Err("Expected unary postfix operator".to_string()),
    }
}

fn parse_binary_operator(tokens: &mut VecDeque<Token>) -> Result<BinaryOperator, String> {
    match tokens.pop_front() {
        Some(Token::Plus) => Ok(BinaryOperator::Add),
        Some(Token::Minus) => Ok(BinaryOperator::Subtract),
        Some(Token::Asterisk) => Ok(BinaryOperator::Multiply),
        Some(Token::Slash) => Ok(BinaryOperator::Divide),
        Some(Token::Percent) => Ok(BinaryOperator::Remainder),
        Some(Token::Ampersand) => Ok(BinaryOperator::BitwiseAnd),
        Some(Token::Pipe) => Ok(BinaryOperator::BitwiseOr),
        Some(Token::Caret) => Ok(BinaryOperator::BitwiseXor),
        Some(Token::LessLess) => Ok(BinaryOperator::ShiftLeft),
        Some(Token::GreaterGreater) => Ok(BinaryOperator::ShiftRight),
        Some(Token::AmpersandAmpersand) => Ok(BinaryOperator::LogicalAnd),
        Some(Token::PipePipe) => Ok(BinaryOperator::LogicalOr),
        Some(Token::EqualEqual) => Ok(BinaryOperator::Equal),
        Some(Token::ExclamationEqual) => Ok(BinaryOperator::NotEqual),
        Some(Token::Less) => Ok(BinaryOperator::LessThan),
        Some(Token::LessEqual) => Ok(BinaryOperator::LessOrEqual),
        Some(Token::Greater) => Ok(BinaryOperator::GreaterThan),
        Some(Token::GreaterEqual) => Ok(BinaryOperator::GreaterOrEqual),
        _ => Err("Expected binary operator".to_string()),
    }
}

fn parse_assignment_operator(tokens: &mut VecDeque<Token>) -> Result<AssignmentOperator, String> {
    match tokens.pop_front() {
        Some(Token::Equal) => Ok(AssignmentOperator::Assign),
        Some(Token::PlusEqual) => Ok(AssignmentOperator::AddAssign),
        Some(Token::MinusEqual) => Ok(AssignmentOperator::SubtractAssign),
        Some(Token::AsteriskEqual) => Ok(AssignmentOperator::MultiplyAssign),
        Some(Token::SlashEqual) => Ok(AssignmentOperator::DivideAssign),
        Some(Token::PercentEqual) => Ok(AssignmentOperator::RemainderAssign),
        Some(Token::AmpersandEqual) => Ok(AssignmentOperator::BitwiseAndAssign),
        Some(Token::PipeEqual) => Ok(AssignmentOperator::BitwiseOrAssign),
        Some(Token::CaretEqual) => Ok(AssignmentOperator::BitwiseXorAssign),
        Some(Token::LessLessEqual) => Ok(AssignmentOperator::ShiftLeftAssign),
        Some(Token::GreaterGreaterEqual) => Ok(AssignmentOperator::ShiftRightAssign),
        _ => Err("Expected assignment operator".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let tokens = vec![
            Token::IntKeyword,
            Token::Identifier("main".to_string()),
            Token::OpenParen,
            Token::VoidKeyword,
            Token::CloseParen,
            Token::OpenBrace,
            Token::ReturnKeyword,
            Token::Constant(42),
            Token::Semicolon,
            Token::CloseBrace,
        ];

        let expected = Program {
            functions: vec![FunctionDeclaration {
                function: Function {
                    identifier: "main".to_string(),
                },
                parameters: vec![],
                body: Some(Block {
                    items: vec![BlockItem::Statement(Statement::Return(
                        Expression::Constant(42),
                    ))],
                }),
            }],
        };

        assert_eq!(parse(&tokens), Ok(expected));
    }

    #[test]
    fn test_parse_error() {
        let tokens = vec![
            Token::IntKeyword,
            Token::Identifier("main".to_string()),
            Token::OpenParen,
            Token::VoidKeyword,
            Token::CloseParen,
            Token::OpenBrace,
            Token::ReturnKeyword,
            Token::Constant(42),
            Token::CloseBrace,
        ];

        assert!(parse(&tokens).is_err());
    }
}
