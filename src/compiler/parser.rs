use std::collections::VecDeque;

use super::{
    ast::{BinaryOperator, Expression, Function, Program, Statement, UnaryOperator},
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

    let body = parse_statement(tokens)?;

    let Some(Token::CloseBrace) = tokens.pop_front() else {
        return Err("Expected close brace".to_string());
    };

    Ok(Function { name, body })
}

fn parse_statement(tokens: &mut VecDeque<Token>) -> Result<Statement, String> {
    let Some(Token::ReturnKeyword) = tokens.pop_front() else {
        return Err("Expected return keyword".to_string());
    };

    let expression = parse_expression(tokens, 0)?;

    let Some(Token::Semicolon) = tokens.pop_front() else {
        return Err("Expected semicolon".to_string());
    };

    Ok(Statement::Return(expression))
}

fn parse_expression(
    tokens: &mut VecDeque<Token>,
    min_precedence: u8,
) -> Result<Expression, String> {
    let mut left = parse_factor(tokens)?;
    while let Some(t) = tokens.front().cloned() {
        let precedence = match t {
            Token::Plus | Token::Minus => 1,
            Token::Asterisk | Token::Slash | Token::Percent => 2,
            _ => break,
        };

        if precedence < min_precedence {
            break;
        }

        let operator = parse_binary_operator(tokens)?;
        let right = parse_expression(tokens, precedence + 1)?;
        left = Expression::Binary(operator, Box::new(left), Box::new(right));
    }
    Ok(left)
}

fn parse_factor(tokens: &mut VecDeque<Token>) -> Result<Expression, String> {
    match tokens.front().cloned() {
        Some(Token::Constant(value)) => {
            tokens.pop_front();
            Ok(Expression::Constant(value))
        }
        Some(Token::Tilde | Token::Minus) => {
            let operator = parse_unary_operator(tokens)?;
            let inner = parse_factor(tokens)?;
            Ok(Expression::Unary(operator, Box::new(inner)))
        }
        Some(Token::OpenParen) => {
            tokens.pop_front();
            let inner = parse_expression(tokens, 0)?;
            let Some(Token::CloseParen) = tokens.pop_front() else {
                return Err("Expected close parenthesis".to_string());
            };
            Ok(inner)
        }
        _ => Err("Expected factor".to_string()),
    }
}

fn parse_unary_operator(tokens: &mut VecDeque<Token>) -> Result<UnaryOperator, String> {
    match tokens.pop_front() {
        Some(Token::Tilde) => Ok(UnaryOperator::Complement),
        Some(Token::Minus) => Ok(UnaryOperator::Negate),
        _ => Err("Expected unary operator".to_string()),
    }
}

fn parse_binary_operator(tokens: &mut VecDeque<Token>) -> Result<BinaryOperator, String> {
    match tokens.pop_front() {
        Some(Token::Plus) => Ok(BinaryOperator::Add),
        Some(Token::Minus) => Ok(BinaryOperator::Subtract),
        Some(Token::Asterisk) => Ok(BinaryOperator::Multiply),
        Some(Token::Slash) => Ok(BinaryOperator::Divide),
        Some(Token::Percent) => Ok(BinaryOperator::Remainder),
        _ => Err("Expected binary operator".to_string()),
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
                body: Statement::Return(Expression::Constant(42)),
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
