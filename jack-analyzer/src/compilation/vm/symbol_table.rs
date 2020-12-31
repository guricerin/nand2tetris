use crate::parse::ast::{self, Ident, Type};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Kind {
    Static, // scope: class
    Field,  // scope: class
    Arg,    // scope: subroutine
    Var,    // scope: subroutine
}

impl fmt::Display for Kind {
    /// 生成する文字列はセグメント名
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Kind::Static => "static",
            Kind::Field => "this",
            Kind::Arg => "argument",
            Kind::Var => "local",
        };
        write!(f, "{}", s)
    }
}

impl From<ast::ClassVarModifier> for Kind {
    fn from(from: ast::ClassVarModifier) -> Self {
        use ast::ClassVarModifier;
        match from {
            ClassVarModifier::Static => Kind::Static,
            ClassVarModifier::Field => Kind::Field,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Record {
    pub ty: Type,
    pub kind: Kind,
    pub id: u64,
}

impl Record {
    pub fn new(ty: Type, kind: Kind, id: u64) -> Self {
        Self { ty, kind, id }
    }
}

#[derive(Debug)]
struct Table {
    table: HashMap<Ident, Record>,
    kind_counter: HashMap<Kind, u64>, // for id
}

impl Table {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            kind_counter: HashMap::new(),
        }
    }
    pub fn insert(&mut self, name: Ident, ty: Type, kind: Kind) {
        let id = self.kind_counter.entry(kind.clone()).or_insert(0);
        let r = Record::new(ty, kind, id.clone());
        let _ = self.table.insert(name, r);
        *id += 1;
    }
    pub fn count(&self, kind: &Kind) -> u64 {
        match self.kind_counter.get(kind) {
            Some(c) => c.clone(),
            None => 0,
        }
    }
    pub fn get(&self, symbol: &Ident) -> Option<&Record> {
        self.table.get(symbol)
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    class_table: Table,
    subroutine_table: Table,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            class_table: Table::new(),
            subroutine_table: Table::new(),
        }
    }

    pub fn start_subroutine(&mut self) {
        self.subroutine_table = Table::new();
    }

    pub fn define(&mut self, name: &Ident, ty: &Type, kind: &Kind) {
        let (name, ty, kind) = (name.clone(), ty.clone(), kind.clone());
        match kind {
            Kind::Static | Kind::Field => {
                self.class_table.insert(name, ty, kind);
            }
            _ => {
                self.subroutine_table.insert(name, ty, kind);
            }
        }
    }

    pub fn varcount(&self, kind: &Kind) -> u64 {
        match kind {
            Kind::Static | Kind::Field => self.class_table.count(kind),
            Kind::Arg | Kind::Var => self.subroutine_table.count(kind),
        }
    }

    pub fn get(&self, symbol: &Ident) -> Option<&Record> {
        let _ = match self.subroutine_table.get(symbol) {
            Some(r) => return Some(r),
            None => (),
        };
        self.class_table.get(symbol)
    }
}
