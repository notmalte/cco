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
    Mov {
        src: Operand,
        dst: Operand,
    },
    Unary {
        op: UnaryOperator,
        dst: Operand,
    },
    Binary {
        op: BinaryOperator,
        src: Operand,
        dst: Operand,
    },
    Cmp {
        src: Operand,
        dst: Operand,
    },
    Idiv(Operand),
    Cdq,
    Sal(Operand),
    Sar(Operand),
    Jmp {
        target: Label,
    },
    JmpCC {
        cc: ConditionCode,
        target: Label,
    },
    SetCC {
        cc: ConditionCode,
        dst: Operand,
    },
    Label(Label),
    AllocateStack(u64),
    Ret,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOperator {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mult,
    And,
    Or,
    Xor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Imm(i64),
    Reg(Reg),
    Pseudo(String),
    Stack(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub identifier: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConditionCode {
    E,
    NE,
    G,
    GE,
    L,
    LE,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reg {
    AX,
    CX,
    DX,
    R10,
    R11,
}
