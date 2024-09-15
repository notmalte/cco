use std::collections::VecDeque;

use super::{
    ast::{
        AssignmentOperator, BinaryOperator, BlockItem, Declaration, Expression, Function, Program,
        Statement, UnaryOperator, Variable,
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
        function_definition: parse_function(tokens)?,
    })
}

fn parse_function(tokens: &mut VecDeque<Token>) -> Result<Function, String> {
    let Some(Token::IntKeyword) = tokens.pop_front() else {
        return Err("Expected int keyword".to_string());
    };

    let Some(Token::Identifier(name)) = tokens.pop_front() else {
        return Err("Expected identifier".to_string());
    };

    let Some(Token::OpenParen) = tokens.pop_front() else {
        return Err("Expected open parenthesis".to_string());
    };

    let Some(Token::VoidKeyword) = tokens.pop_front() else {
        return Err("Expected void keyword".to_string());
    };

    let Some(Token::CloseParen) = tokens.pop_front() else {
        return Err("Expected close parenthesis".to_string());
    };

    let Some(Token::OpenBrace) = tokens.pop_front() else {
        return Err("Expected open brace".to_string());
    };

    let mut body = vec![];

    while let Some(t) = tokens.front() {
        if t == &Token::CloseBrace {
            break;
        }

        body.push(parse_block_item(tokens)?);
    }

    let Some(Token::CloseBrace) = tokens.pop_front() else {
        return Err("Expected close brace".to_string());
    };

    Ok(Function { name, body })
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

    let variable = Variable { identifier };

    if let Some(Token::Equal) = tokens.front() {
        tokens.pop_front();
        let expression = parse_expression(tokens, 0)?;

        let Some(Token::Semicolon) = tokens.pop_front() else {
            return Err("Expected semicolon".to_string());
        };

        Ok(Declaration {
            variable,
            initializer: Some(expression),
        })
    } else {
        let Some(Token::Semicolon) = tokens.pop_front() else {
            return Err("Expected semicolon".to_string());
        };

        Ok(Declaration {
            variable,
            initializer: None,
        })
    }
}

fn parse_statement(tokens: &mut VecDeque<Token>) -> Result<Statement, String> {
    match tokens.front() {
        Some(Token::Semicolon) => parse_null_statement(tokens),
        Some(Token::ReturnKeyword) => parse_return_statement(tokens),
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
            Token::PipePipe => 2,
            Token::AmpersandAmpersand => 3,
            Token::Pipe => 4,
            Token::Caret => 5,
            Token::Ampersand => 6,
            Token::EqualEqual | Token::ExclamationEqual => 7,
            Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual => 8,
            Token::LessLess | Token::GreaterGreater => 9,
            Token::Plus | Token::Minus => 10,
            Token::Asterisk | Token::Slash | Token::Percent => 11,
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
    let factor = match tokens.front().cloned() {
        Some(Token::Constant(value)) => {
            tokens.pop_front();
            Expression::Constant(value)
        }
        Some(Token::Identifier(identifier)) => {
            tokens.pop_front();
            Expression::Variable(Variable { identifier })
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

    Ok(match tokens.front() {
        Some(Token::PlusPlus | Token::MinusMinus) => {
            let op = parse_unary_postfix_operator(tokens)?;
            Expression::Unary {
                op,
                expr: Box::new(factor),
            }
        }
        _ => factor,
    })
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
            function_definition: Function {
                name: "main".to_string(),
                body: vec![BlockItem::Statement(Statement::Return(
                    Expression::Constant(42),
                ))],
            },
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
