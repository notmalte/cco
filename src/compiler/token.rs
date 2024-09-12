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
    Constant(i64),

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
    /// `~`
    Tilde,
    /// `-`
    Minus,
    /// `+`
    Plus,
    /// `*`
    Asterisk,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `&`
    Ampersand,
    /// `|`
    Pipe,
    /// `^`
    Caret,
    /// `!`
    Exclamation,
    /// `<`
    Less,
    /// `>`
    Greater,
    /// `=`
    Equal,

    /// `--`
    MinusMinus,
    /// `<<`
    LessLess,
    /// `>>`
    GreaterGreater,
    /// `&&`
    AmpersandAmpersand,
    /// `||`
    PipePipe,
    /// `==`
    EqualEqual,
    /// `!=`
    ExclamationEqual,
    /// `<=`
    LessEqual,
    /// `>=`
    GreaterEqual,
}
