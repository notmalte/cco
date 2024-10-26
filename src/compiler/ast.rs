#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Long,
    Function {
        return_type: Box<Type>,
        parameters: Vec<Type>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Variable(VariableDeclaration),
    Function(FunctionDeclaration),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclaration {
    pub variable: Variable,
    pub initializer: Option<Expression>,
    pub ty: Type,
    pub storage_class: Option<StorageClass>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    pub function: Function,
    pub parameters: Vec<Variable>,
    pub body: Option<Block>,
    pub ty: Type,
    pub storage_class: Option<StorageClass>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StorageClass {
    Static,
    Extern,
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
    Break(Option<LoopOrSwitchLabel>),
    Continue(Option<LoopLabel>),
    While {
        condition: Expression,
        body: Box<Statement>,
        label: Option<LoopLabel>,
    },
    DoWhile {
        body: Box<Statement>,
        condition: Expression,
        label: Option<LoopLabel>,
    },
    For {
        initializer: Option<ForInitializer>,
        condition: Option<Expression>,
        post: Option<Expression>,
        body: Box<Statement>,
        label: Option<LoopLabel>,
    },
    Switch {
        expression: Expression,
        body: Box<Statement>,
        cases: Option<SwitchCases>,
        label: Option<SwitchLabel>,
    },
    Case {
        expression: Expression,
        body: Box<Statement>,
        label: Option<SwitchCaseLabel>,
    },
    Default {
        body: Box<Statement>,
        label: Option<SwitchCaseLabel>,
    },
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForInitializer {
    VariableDeclaration(VariableDeclaration),
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Constant {
        c: Constant,
        ty: Option<Type>,
    },
    Variable {
        v: Variable,
        ty: Option<Type>,
    },
    Cast {
        target_ty: Type,
        expr: Box<Expression>,
        ty: Option<Type>,
    },
    Unary {
        op: UnaryOperator,
        expr: Box<Expression>,
        ty: Option<Type>,
    },
    Binary {
        op: BinaryOperator,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        ty: Option<Type>,
    },
    Assignment {
        op: AssignmentOperator,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        ty: Option<Type>,
    },
    Conditional {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
        ty: Option<Type>,
    },
    FunctionCall {
        function: Function,
        arguments: Vec<Expression>,
        ty: Option<Type>,
    },
}

impl Expression {
    pub fn ty(&self) -> Option<Type> {
        match self {
            Expression::Constant { ty, .. } => ty.clone(),
            Expression::Variable { ty, .. } => ty.clone(),
            Expression::Cast { ty, .. } => ty.clone(),
            Expression::Unary { ty, .. } => ty.clone(),
            Expression::Binary { ty, .. } => ty.clone(),
            Expression::Assignment { ty, .. } => ty.clone(),
            Expression::Conditional { ty, .. } => ty.clone(),
            Expression::FunctionCall { ty, .. } => ty.clone(),
        }
    }
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

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchLabel {
    pub identifier: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoopOrSwitchLabel {
    Loop(LoopLabel),
    Switch(SwitchLabel),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub identifier: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCaseLabel {
    pub identifier: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCases {
    pub cases: Vec<(Expression, SwitchCaseLabel)>,
    pub default: Option<SwitchCaseLabel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constant {
    ConstantInt(i32),
    ConstantLong(i64),
}
