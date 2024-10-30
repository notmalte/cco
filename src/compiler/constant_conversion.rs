use crate::compiler::ast::{Constant, Type};

pub fn convert_constant_to_type(c: &Constant, ty: &Type) -> Constant {
    match ty {
        Type::Int => match c {
            Constant::ConstantInt(n) => Constant::ConstantInt(*n),
            Constant::ConstantLong(n) => Constant::ConstantInt(*n as i32),
        },
        Type::Long => match c {
            Constant::ConstantInt(n) => Constant::ConstantLong(*n as i64),
            Constant::ConstantLong(n) => Constant::ConstantLong(*n),
        },
        Type::Function { .. } => unreachable!(),
    }
}
