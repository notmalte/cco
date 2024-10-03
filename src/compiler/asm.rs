#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub function_definitions: Vec<FunctionDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition {
    pub function: Function,
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
    DeallocateStack(u64),
    Push(Operand),
    Call(Function),
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
    Stack(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub identifier: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConditionCode {
    /// Equal
    E,
    /// Not Equal
    NE,
    /// Greater
    G,
    /// Greater or Equal
    GE,
    /// Less
    L,
    /// Less or Equal
    LE,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reg {
    AX,
    CX,
    DX,
    DI,
    SI,
    R8,
    R9,
    R10,
    R11,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub identifier: String,
}
