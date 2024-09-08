use regex::Regex;

use super::token::Token;

fn find_first_token(s: &str) -> Option<(Token, &str)> {
    if s.is_empty() {
        return None;
    }

    if let Some(m) = Regex::new(r"^[a-zA-Z_]\w*\b").unwrap().find(s) {
        let ms = m.as_str();
        let rest = &s[m.end()..];

        let t = match ms {
            "void" => Token::VoidKeyword,
            "int" => Token::IntKeyword,
            "return" => Token::ReturnKeyword,
            _ => Token::Identifier(ms.to_string()),
        };

        return Some((t, rest));
    }

    if let Some(m) = Regex::new(r"^\d+\b").unwrap().find(s) {
        let ms = m.as_str();
        let rest = &s[m.end()..];

        let t = Token::IntLiteral(ms.parse().unwrap());

        return Some((t, rest));
    }

    let single_char_tokens = [
        ('(', Token::OpenParen),
        (')', Token::CloseParen),
        ('{', Token::OpenBrace),
        ('}', Token::CloseBrace),
        (';', Token::Semicolon),
    ];

    let first_char = s.chars().next().unwrap();

    single_char_tokens
        .iter()
        .find_map(|(c, t)| (*c == first_char).then(|| (t.clone(), &s[1..])))
}

pub fn tokenize(s: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut rest = s.trim_start();

    while !rest.is_empty() {
        if let Some((t, r)) = find_first_token(rest) {
            tokens.push(t);
            rest = r.trim_start();
        } else {
            return Err(format!("Could not tokenize: {}", rest));
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        assert_eq!(tokenize(""), Ok(vec![]));
    }

    #[test]
    fn test_single_tokens() {
        let test_cases = vec![
            ("void", Token::VoidKeyword),
            ("int", Token::IntKeyword),
            ("return", Token::ReturnKeyword),
            ("42", Token::IntLiteral(42)),
            ("(", Token::OpenParen),
            (")", Token::CloseParen),
            ("{", Token::OpenBrace),
            ("}", Token::CloseBrace),
            (";", Token::Semicolon),
            ("main", Token::Identifier("main".to_string())),
            ("foo", Token::Identifier("foo".to_string())),
            ("_bar", Token::Identifier("_bar".to_string())),
            ("baz123", Token::Identifier("baz123".to_string())),
        ];

        for (input, expected_token) in test_cases {
            assert_eq!(tokenize(input), Ok(vec![expected_token.clone()]),);
        }
    }

    #[test]
    fn test_multiple_tokens() {
        let input = "int main(void) { return 2; }";
        let expected = Ok(vec![
            Token::IntKeyword,
            Token::Identifier("main".to_string()),
            Token::OpenParen,
            Token::VoidKeyword,
            Token::CloseParen,
            Token::OpenBrace,
            Token::ReturnKeyword,
            Token::IntLiteral(2),
            Token::Semicolon,
            Token::CloseBrace,
        ]);

        assert_eq!(tokenize(input), expected);
    }

    #[test]
    fn test_valid_tokens_invalid_ast() {
        let input = "int void main((void() {;{ return return; void }";
        let expected = Ok(vec![
            Token::IntKeyword,
            Token::VoidKeyword,
            Token::Identifier("main".to_string()),
            Token::OpenParen,
            Token::OpenParen,
            Token::VoidKeyword,
            Token::OpenParen,
            Token::CloseParen,
            Token::OpenBrace,
            Token::Semicolon,
            Token::OpenBrace,
            Token::ReturnKeyword,
            Token::ReturnKeyword,
            Token::Semicolon,
            Token::VoidKeyword,
            Token::CloseBrace,
        ]);

        assert_eq!(tokenize(input), expected);
    }

    #[test]
    fn test_invalid_input() {
        let test_cases = vec!["@", "\\", "int main(void) { return 2; } @abc", "123abc"];

        for input in test_cases {
            assert!(tokenize(input).is_err());
        }
    }

    #[test]
    fn test_whitespace() {
        let input = "   int  main  (  void  )  {\n\n\treturn 2;\n}";
        let expected = Ok(vec![
            Token::IntKeyword,
            Token::Identifier("main".to_string()),
            Token::OpenParen,
            Token::VoidKeyword,
            Token::CloseParen,
            Token::OpenBrace,
            Token::ReturnKeyword,
            Token::IntLiteral(2),
            Token::Semicolon,
            Token::CloseBrace,
        ]);

        assert_eq!(tokenize(input), expected);
    }
}
