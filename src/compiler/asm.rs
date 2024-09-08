#[derive(Debug, PartialEq)]
pub struct Program {
    pub function_definition: Function,
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Mov { src: Operand, dst: Operand },
    Ret,
}

#[derive(Debug, PartialEq)]
pub enum Operand {
    Imm(i32),
    Register,
}
