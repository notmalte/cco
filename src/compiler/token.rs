#[derive(Debug, Clone, PartialEq)]
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
