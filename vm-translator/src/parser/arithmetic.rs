/// スタック算術コマンド
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arithmetic {
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
}
