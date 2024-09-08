use std::collections::VecDeque;

use super::{
    ast::{Expression, Function, Program, Statement},
    token::Token,
};

fn parse_program(tokens: &mut VecDeque<Token>) -> Result<Program, String> {
    Ok(Program {
        function_definition: parse_function(tokens)?,
    })
}

fn parse_function(tokens: &mut VecDeque<Token>) -> Result<Function, String> {
    match tokens.pop_front() {
        Some(Token::IntKeyword) => {}
        _ => return Err("Expected int keyword".to_string()),
    }

    let name = match tokens.pop_front() {
        Some(Token::Identifier(name)) => name,
        _ => return Err("Expected identifier".to_string()),
    };

    match tokens.pop_front() {
        Some(Token::OpenParen) => {}
        _ => return Err("Expected open parenthesis".to_string()),
    }

    match tokens.pop_front() {
        Some(Token::VoidKeyword) => {}
        _ => return Err("Expected void keyword".to_string()),
    }

    match tokens.pop_front() {
        Some(Token::CloseParen) => {}
        _ => return Err("Expected close parenthesis".to_string()),
    }

    match tokens.pop_front() {
        Some(Token::OpenBrace) => {}
        _ => return Err("Expected open brace".to_string()),
    }

    let body = parse_statement(tokens)?;

    match tokens.pop_front() {
        Some(Token::CloseBrace) => {}
        _ => return Err("Expected close brace".to_string()),
    }

    Ok(Function { name, body })
}

fn parse_statement(tokens: &mut VecDeque<Token>) -> Result<Statement, String> {
    match tokens.pop_front() {
        Some(Token::ReturnKeyword) => {}
        _ => return Err("Expected return keyword".to_string()),
    }

    let expression = parse_expression(tokens)?;

    match tokens.pop_front() {
        Some(Token::Semicolon) => {}
        _ => return Err("Expected semicolon".to_string()),
    }

    Ok(Statement::Return(expression))
}

fn parse_expression(tokens: &mut VecDeque<Token>) -> Result<Expression, String> {
    match tokens.pop_front() {
        Some(Token::IntLiteral(value)) => Ok(Expression::IntLiteral(value)),
        _ => Err("Expected int literal".to_string()),
    }
}

pub fn parse(tokens: &[Token]) -> Result<Program, String> {
    let mut tokens = VecDeque::from_iter(tokens.iter().cloned());

    let program = parse_program(&mut tokens)?;

    if !tokens.is_empty() {
        return Err("Expected EOF".to_string());
    }

    Ok(program)
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
            Token::IntLiteral(42),
            Token::Semicolon,
            Token::CloseBrace,
        ];

        let expected = Program {
            function_definition: Function {
                name: "main".to_string(),
                body: Statement::Return(Expression::IntLiteral(42)),
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
            Token::IntLiteral(42),
            Token::CloseBrace,
        ];

        assert!(parse(&tokens).is_err());
    }
}
