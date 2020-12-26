#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Func {
    /// 関数定義
    Func { name: String, paramc: u16 },
    /// 関数呼び出し
    Call { name: String, argc: u16 },
    /// return
    Return,
}
