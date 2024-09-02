use regex::Regex;

#[derive(Debug, Clone)]
pub enum Token {
    Identifier(String),
    VoidKeyword,
    IntKeyword,
    ReturnKeyword,
    IntLiteral(i32),
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semicolon,
}

fn find_first_token(s: &str) -> Option<(Token, &str)> {
    if s.is_empty() {
        return None;
    }

    if let Some(m) = Regex::new(r"^[a-zA-Z_]\w*\b").unwrap().find(&s) {
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

    if let Some(m) = Regex::new(r"^\d+\b").unwrap().find(&s) {
        let ms = m.as_str();
        let rest = &s[m.end()..];

        let t = Token::IntLiteral(ms.parse().unwrap());

        return Some((t, rest));
    }

    let single_char_tokens = vec![
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

pub fn tokenize(s: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut rest = s.trim_start();

    while !rest.is_empty() {
        if let Some((t, r)) = find_first_token(&rest) {
            tokens.push(t);
            rest = r.trim_start();
        } else {
            panic!("Could not tokenize: {:?}", rest);
        }
    }

    tokens
}
