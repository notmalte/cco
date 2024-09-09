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
    Constant(u64),
    Unary(UnaryOperator, Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Complement,
    Negate,
}
