use std::collections::VecDeque;

use super::{
    ast::{Expression, Function, Program, Statement, UnaryOperator},
    token::Token,
};

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

    let expression = parse_expression(tokens)?;

    let Some(Token::Semicolon) = tokens.pop_front() else {
        return Err("Expected semicolon".to_string());
    };

    Ok(Statement::Return(expression))
}

fn parse_expression(tokens: &mut VecDeque<Token>) -> Result<Expression, String> {
    match tokens.front().cloned() {
        Some(Token::IntLiteral(value)) => {
            tokens.pop_front();
            return Ok(Expression::IntLiteral(value));
        }
        Some(Token::Tilde | Token::Minus) => {
            let operator = parse_unary_operator(tokens)?;
            let inner = Box::new(parse_expression(tokens)?);
            return Ok(Expression::Unary(operator, inner));
        }
        Some(Token::OpenParen) => {
            tokens.pop_front();
            let inner = Box::new(parse_expression(tokens)?);
            let Some(Token::CloseParen) = tokens.pop_front() else {
                return Err("Expected close parenthesis".to_string());
            };
            return Ok(*inner);
        }
        _ => return Err("Expected expression".to_string()),
    }
}

fn parse_unary_operator(tokens: &mut VecDeque<Token>) -> Result<UnaryOperator, String> {
    match tokens.pop_front() {
        Some(Token::Tilde) => Ok(UnaryOperator::Complement),
        Some(Token::Minus) => Ok(UnaryOperator::Negate),
        _ => Err("Expected unary operator".to_string()),
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
