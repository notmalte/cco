#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub function_definition: Function,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub body: Statement,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Return(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Constant(i64),
    Unary {
        op: UnaryOperator,
        expr: Box<Expression>,
    },
    Binary {
        op: BinaryOperator,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Complement,
    Negate,
}

#[derive(Debug, Clone, PartialEq)]
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
}
