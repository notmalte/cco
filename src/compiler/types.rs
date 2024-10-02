#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Int,
    Function { parameter_count: usize },
}
