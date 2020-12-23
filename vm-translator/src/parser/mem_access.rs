use super::segment::*;

/// メモリアクセスコマンド
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemAccess {
    Push(Segment, u16),
    Pop(Segment, u16),
}

impl MemAccess {
    pub fn push(seg: Segment, index: u16) -> Self {
        Self::Push(seg, index)
    }
    pub fn pop(seg: Segment, index: u16) -> Self {
        Self::Pop(seg, index)
    }
}
