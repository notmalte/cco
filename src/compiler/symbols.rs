use crate::compiler::ast::Type;

use std::collections::{hash_map::Iter, HashMap};

#[derive(Debug, Clone)]
pub enum SymbolAttributes {
    Function {
        defined: bool,
        global: bool,
    },
    Static {
        initial: SymbolInitialValue,
        global: bool,
    },
    Local,
}

#[derive(Debug, Clone, Copy)]
pub enum SymbolInitialValue {
    Tentative,
    Initial(i64),
    None,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub ty: Type,
    pub attrs: SymbolAttributes,
}

pub struct SymbolTable {
    entries: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, identifier: &str) -> Option<&Symbol> {
        self.entries.get(identifier)
    }

    pub fn insert(&mut self, identifier: String, entry: Symbol) -> Option<Symbol> {
        self.entries.insert(identifier, entry)
    }

    pub fn iter(&self) -> Iter<String, Symbol> {
        self.entries.iter()
    }
}
