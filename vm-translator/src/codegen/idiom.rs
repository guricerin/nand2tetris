use crate::parser::segment::*;

pub static INITIALIZE: &'static str = r#"// initialize
@256 // RAM[0] (SP) = 256
D=A
@SP
M=D
"#;

// 汎用的なレジスタとしてVM側で自由に扱えるRAMアドレス
static GENERIC_REG_ADDR_0: &'static str = "R13";
// static GENERIC_REG_ADDR_1: &'static str = "R14";
// static GENERIC_REG_ADDR_2: &'static str = "R15";

/// SP-- の後に *(SP-1) = expr
pub fn binary_op(expr: &str) -> String {
    // スタックからrightをポップ
    // スタックの頂点(left)を left op right に書き換える
    // A=A-1 は、RAM[@SP]を-1するわけではない
    format!(
        r#"// binary op
@SP
AM=M-1  // SP--
D=M     // D = *SP
A=A-1   // ptr = SP - 1
M={}      // *(ptr) = *(ptr) op D
"#,
        expr
    )
}

/// *(SP-1) = op *SP
pub fn unary_op(op: &str) -> String {
    format!(
        r#"// unary op
@SP     // ptr = SP
A=M-1   // ptr = ptr - 1
M={}M   // *(ptr) = op *(ptr)
"#,
        op
    )
}

/// コールする前に、プッシュしたいデータをDに格納すること
pub fn push_from_d() -> String {
    // スタックにプッシュ
    // @SPの参照先にDを入れた後、@SP自体をインクリメント
    let code = r#"// push from D
@SP     // *SP = D
A=M
M=D
@SP     // SP++
M=M+1
"#;
    code.to_owned()
}

/// ポップしたデータをDレジスタに格納
pub fn pop_to_d() -> String {
    let code = r#"// pop to D
@SP     //
AM=M-1  // SP--
D=M     // D = *SP
"#;
    code.to_owned()
}

pub fn push_from_named_segment(seg_label: &str, index: u16) -> String {
    format!(
        r#"//pop to named segment
{0}
@{1}    // fetch from R13
D=M
{2}
"#,
        save_addr(seg_label, index),
        GENERIC_REG_ADDR_0,
        push_from_d()
    )
}

// todo: 消せ
fn ram_index(seg: &Segment) -> u16 {
    use Segment::*;
    match seg {
        Pointer => 3,
        Temp => 5,
        _ => unreachable!(),
    }
}

pub fn push_from_unnamed_segment(seg: &Segment, index: u16) -> String {
    format!(
        r#"//pop to named segment
{0}
@{1}    // fetch from R13
D=M
{2}
"#,
        save_ram_addr(ram_index(seg) + index),
        GENERIC_REG_ADDR_0,
        push_from_d()
    )
}

fn save_ram_addr(index: u16) -> String {
    format!(
        r#"//save ptr or temp segment index
@R{0}
D=M
@{1}
M=D
"#,
        index, GENERIC_REG_ADDR_0
    )
}

pub fn pop_to_named_segment(seg_label: &str, index: u16) -> String {
    format!(
        "//pop to named segment\n{}{}{}",
        save_addr(seg_label, index),
        pop_to_d(),
        set_d_to_addr()
    )
}

pub fn pop_to_unnamed_segment(seg: &Segment, index: u16) -> String {
    format!(
        "//pop to temp segment\n{}{}{}",
        save_ram_addr(ram_index(seg) + index),
        pop_to_d(),
        set_d_to_addr()
    )
}

/// segment[index]をR13に保存
pub fn save_addr(seg_label: &str, index: u16) -> String {
    format!(
        r#"// save segment index
@{0}
D=M
@{1}
D=D+A
@{2}
M=D
"#,
        seg_label, index, GENERIC_REG_ADDR_0
    )
}

pub fn set_d_to_addr() -> String {
    format!(
        r#"//set D to segment[index]
@{0}
M=A
M=D
"#,
        GENERIC_REG_ADDR_0
    )
}
