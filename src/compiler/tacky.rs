#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub function_definition: Function,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Return(Value),
    Unary {
        op: UnaryOperator,
        src: Value,
        dst: Variable,
    },
    Binary {
        op: BinaryOperator,
        lhs: Value,
        rhs: Value,
        dst: Variable,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Constant(i64),
    Variable(Variable),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub identifier: String,
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
