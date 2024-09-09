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
    Mov { src: Operand, dst: Operand },
    Unary(UnaryOperator, Operand),
    AllocateStack(u64),
    Ret,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Imm(u64),
    Reg(Reg),
    Pseudo(String),
    Stack(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Reg {
    AX,
    R10,
}
