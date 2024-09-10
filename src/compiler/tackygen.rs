use super::{ast, tacky};

pub struct TackyGen {
    counter: usize,
}

impl TackyGen {
    fn fresh_variable(&mut self) -> tacky::Variable {
        let name = format!("tmp.{}", self.counter);
        self.counter += 1;

        tacky::Variable { identifier: name }
    }

    fn handle_program(&mut self, program: &ast::Program) -> tacky::Program {
        tacky::Program {
            function_definition: self.handle_function(&program.function_definition),
        }
    }

    fn handle_function(&mut self, function: &ast::Function) -> tacky::Function {
        tacky::Function {
            name: function.name.clone(),
            instructions: self.handle_statement(&function.body),
        }
    }

    fn handle_statement(&mut self, statement: &ast::Statement) -> Vec<tacky::Instruction> {
        match statement {
            ast::Statement::Return(expr) => {
                let mut ins = vec![];
                let val = self.handle_expression(&mut ins, expr);
                ins.push(tacky::Instruction::Return(val));

                ins
            }
        }
    }

    fn handle_expression(
        &mut self,
        ins: &mut Vec<tacky::Instruction>,
        expr: &ast::Expression,
    ) -> tacky::Value {
        match expr {
            ast::Expression::Constant(value) => tacky::Value::Constant(*value),
            ast::Expression::Unary(op, inner) => {
                let src = self.handle_expression(ins, inner);
                let dst = self.fresh_variable();
                let op = Self::handle_unary_operator((*op).clone());

                ins.push(tacky::Instruction::Unary {
                    op,
                    src,
                    dst: dst.clone(),
                });

                tacky::Value::Variable(dst)
            }
            ast::Expression::Binary(_, _, _) => todo!(),
        }
    }

    fn handle_unary_operator(op: ast::UnaryOperator) -> tacky::UnaryOperator {
        match op {
            ast::UnaryOperator::Negate => tacky::UnaryOperator::Negate,
            ast::UnaryOperator::Complement => tacky::UnaryOperator::Complement,
        }
    }
}

pub fn generate(program: &ast::Program) -> tacky::Program {
    let mut tg = TackyGen { counter: 0 };

    tg.handle_program(program)
}
