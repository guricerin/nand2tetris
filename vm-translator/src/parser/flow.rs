#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Flow {
    /// ラベル宣言
    Label(String),
    /// 無条件分岐
    Goto(String),
    /// 条件付き分岐
    IfGoto(String),
}
