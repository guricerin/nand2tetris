use super::segment::*;

/// メモリアクセスコマンド
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemAccess {
    Push(Segment, u16),
    Pop(Segment, u16),
}
