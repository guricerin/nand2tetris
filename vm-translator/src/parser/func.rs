#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Func {
    /// function func-name local-var-num
    Func(String, u32),
    /// call func-name args-num
    Call(String, u32),
    /// return
    Return,
}
