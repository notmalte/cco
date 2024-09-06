use super::lexer::Token;

trait Parsable {
    fn parse(tokens: &[Token]) -> Result<Self, String>
    where
        Self: Sized;
}

#[derive(Debug, PartialEq)]
pub struct Program {
    function_definition: Function,
}

impl Parsable for Program {
    fn parse(tokens: &[Token]) -> Result<Self, String> {
        Ok(Program {
            function_definition: Function::parse(tokens)?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Function {
    name: String,
    body: Statement,
}

impl Parsable for Function {
    fn parse(tokens: &[Token]) -> Result<Self, String> {
        let rest = match tokens {
            [Token::IntKeyword, rest @ ..] => rest,
            _ => return Err("Expected int keyword".to_string()),
        };

        let (name, rest) = match rest {
            [Token::Identifier(name), rest @ ..] => (name.clone(), rest),
            _ => return Err("Expected identifier".to_string()),
        };

        let rest = match rest {
            [Token::OpenParen, rest @ ..] => rest,
            _ => return Err("Expected open parenthesis".to_string()),
        };

        let rest = match rest {
            [Token::VoidKeyword, rest @ ..] => rest,
            _ => return Err("Expected void keyword".to_string()),
        };

        let rest = match rest {
            [Token::CloseParen, rest @ ..] => rest,
            _ => return Err("Expected close parenthesis".to_string()),
        };

        let rest = match rest {
            [Token::OpenBrace, rest @ ..] => rest,
            _ => return Err("Expected open brace".to_string()),
        };

        let rest = match rest {
            [rest @ .., Token::CloseBrace] => rest,
            _ => return Err("Expected close brace".to_string()),
        };

        let body = Statement::parse(rest)?;

        Ok(Function { name, body })
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Return(Expression),
}

impl Parsable for Statement {
    fn parse(tokens: &[Token]) -> Result<Self, String> {
        let rest = match tokens {
            [Token::ReturnKeyword, rest @ ..] => rest,
            _ => return Err("Expected return keyword".to_string()),
        };

        let rest = match rest {
            [rest @ .., Token::Semicolon] => rest,
            _ => return Err("Expected semicolon".to_string()),
        };

        let expression = Expression::parse(rest)?;

        Ok(Statement::Return(expression))
    }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    IntLiteral(i32),
}

impl Parsable for Expression {
    fn parse(tokens: &[Token]) -> Result<Self, String> {
        let value = match tokens {
            [Token::IntLiteral(value)] => value,
            _ => return Err("Expected int literal".to_string()),
        };

        Ok(Expression::IntLiteral(*value))
    }
}

pub fn parse(tokens: &[Token]) -> Result<Program, String> {
    Program::parse(tokens)
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

        assert!(parse(&tokens).is_err(),);
    }
}
