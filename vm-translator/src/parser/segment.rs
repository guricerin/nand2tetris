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

impl Segment {
    pub fn name(&self) -> Option<String> {
        use Segment::*;

        let name = match self {
            Arg => "ARG",
            Local => "LCL",
            This => "THIS",
            That => "THAT",
            _ => return None,
        };
        Some(name.to_string())
    }

    pub fn ram_index(&self) -> Option<u16> {
        use Segment::*;

        match self {
            Pointer => Some(3),
            Temp => Some(5),
            _ => None,
        }
    }
}
