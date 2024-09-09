#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// e.g. `main`
    Identifier(String),

    /// `void`
    VoidKeyword,
    /// `int`
    IntKeyword,
    /// `return`
    ReturnKeyword,

    /// e.g. `42`
    Constant(i32),

    /// `~`
    Tilde,
    /// `-`
    Minus,
    /// `--`
    MinusMinus,

    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
    /// `{`
    OpenBrace,
    /// `}`
    CloseBrace,
    /// `;`
    Semicolon,
}
