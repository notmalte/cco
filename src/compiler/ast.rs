#[derive(Debug, PartialEq)]
pub struct Program {
    pub function_definition: Function,
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub body: Statement,
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Return(Expression),
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    IntLiteral(i32),
    Unary(UnaryOperator, Box<Expression>),
}

#[derive(Debug, PartialEq)]
pub enum UnaryOperator {
    Complement,
    Negate,
}
