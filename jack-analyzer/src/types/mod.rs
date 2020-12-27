use std::fmt;

/// ソースコード中の位置
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Loc {
    row: usize,
    col: usize,
}

impl Loc {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Line: {}, Col: {}", self.row, self.col)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Annot<T> {
    pub value: T,
    pub loc: Loc,
}

impl<T> Annot<T> {
    pub fn new(value: T, loc: Loc) -> Self {
        Self { value, loc }
    }
}
