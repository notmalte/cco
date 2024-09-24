#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub function_definition: Function,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub items: Vec<BlockItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockItem {
    Statement(Statement),
    Declaration(Declaration),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Return(Expression),
    Expression(Expression),
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    Goto(Label),
    Labeled(Label, Box<Statement>),
    Compound(Block),
    Break(LoopLabel),
    Continue(LoopLabel),
    While {
        condition: Expression,
        body: Box<Statement>,
        label: LoopLabel,
    },
    DoWhile {
        body: Box<Statement>,
        condition: Expression,
        label: LoopLabel,
    },
    For {
        initializer: Option<ForInitializer>,
        condition: Option<Expression>,
        post: Option<Expression>,
        body: Box<Statement>,
        label: LoopLabel,
    },
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForInitializer {
    Declaration(Declaration),
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub variable: Variable,
    pub initializer: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Constant(i64),
    Variable(Variable),
    Unary {
        op: UnaryOperator,
        expr: Box<Expression>,
    },
    Binary {
        op: BinaryOperator,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Assignment {
        op: AssignmentOperator,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Conditional {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOperator {
    Complement,
    Negate,
    Not,
    PrefixIncrement,
    PrefixDecrement,
    PostfixIncrement,
    PostfixDecrement,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,
    LogicalAnd,
    LogicalOr,
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub identifier: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    RemainderAssign,
    BitwiseAndAssign,
    BitwiseOrAssign,
    BitwiseXorAssign,
    ShiftLeftAssign,
    ShiftRightAssign,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub identifier: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopLabel {
    pub identifier: String,
}

impl LoopLabel {
    pub fn tbd() -> Self {
        Self {
            identifier: "TO_BE_DEFINED".to_string(),
        }
    }
}
