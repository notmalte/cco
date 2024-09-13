#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub function_definition: Function,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub body: Vec<BlockItem>,
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
    Null,
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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOperator {
    Complement,
    Negate,
    Not,
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
