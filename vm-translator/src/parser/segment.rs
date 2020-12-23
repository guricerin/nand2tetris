#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    Arg,
    Local,
    Static,
    Constant,
    This,
    That,
    Pointer,
    Temp,
}
