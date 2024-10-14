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
    /// `if`
    IfKeyword,
    /// `else`
    ElseKeyword,
    /// `goto`
    GotoKeyword,
    /// `do`
    DoKeyword,
    /// `while`
    WhileKeyword,
    /// `for`
    ForKeyword,
    /// `break`
    BreakKeyword,
    /// `continue`
    ContinueKeyword,
    /// `static`
    StaticKeyword,
    /// `extern`
    ExternKeyword,
    /// `switch`
    SwitchKeyword,
    /// `case`
    CaseKeyword,
    /// `default`
    DefaultKeyword,

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
    /// `?`
    Question,
    /// `:`
    Colon,
    /// `,`
    Comma,

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
    /// `+=`
    PlusEqual,
    /// `-=`
    MinusEqual,
    /// `*=`
    AsteriskEqual,
    /// `/=`
    SlashEqual,
    /// `%=`
    PercentEqual,
    /// `&=`
    AmpersandEqual,
    /// `|=`
    PipeEqual,
    /// `^=`
    CaretEqual,
    /// `--`
    MinusMinus,
    /// `++`
    PlusPlus,

    /// `<<=`
    LessLessEqual,
    /// `>>=`
    GreaterGreaterEqual,
}
