use super::vm_bool::VmBool;

pub use fork::*;
pub use operate::*;
pub use stack_pop::*;
pub use stack_push::*;

pub static INITIALIZE: &'static str = r#"// initialize
@256 // RAM[0] (SP) = 256
D=A
@SP
M=D
"#;

/// 汎用的なレジスタとしてVM側で自由に扱えるRAMアドレス
/// ただしDのようにコマンド一発でデータを格納できるわけではない
static GENERIC_REG_ADDR_0: &'static str = "R13";
// static GENERIC_REG_ADDR_1: &'static str = "R14";
// static GENERIC_REG_ADDR_2: &'static str = "R15";

pub mod operate {
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
}

pub mod fork {
    use super::*;

    pub fn jump(jmp: &str, true_label: &str, false_label: &str, break_label: &str) -> String {
        format!(
            r#"// jump
{0}
A=A-1 // ptr--
D=M-D // lhs - rhs
@{1}
D;{2} // compare D to 0: true -> jump to TRUE false -> jump to FALSE
({3}) // FALSE block
D={4} // D = false
@{6}
0;JMP // jump to BREAK
({1}) // TRUE block
D={5} // D = true
({6}) // BREAK
@SP
M=M-1 // SP-- (lhsが格納されていたアドレスに条件式の結果を突っ込むため)
{7}
"#,
            pop_to_d(),
            true_label,
            jmp,
            false_label,
            VmBool::False as i8,
            VmBool::True as i8,
            break_label,
            push_from_d()
        )
    }
}

pub mod stack_push {
    use super::*;

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

    /// for local, argument, this, that segment
    pub fn push_from_named_segment(seg_name: &str, index: u16) -> String {
        format!(
            r#"/// push from named segment
{0}
@{1}    // fetch from R13
A=M
D=M
{2}
"#,
            save_addr(seg_name, index),
            GENERIC_REG_ADDR_0,
            push_from_d()
        )
    }

    /// for pointer or temp segment
    pub fn push_from_unnamed_segment(r_name: &str) -> String {
        format!(
            r#"/// push from unnamed segment
@{0}
D=M
{1}
        "#,
            r_name,
            push_from_d()
        )
    }
}

pub mod stack_pop {
    use super::*;

    /// ポップしたデータをDレジスタに格納
    pub fn pop_to_d() -> String {
        let code = r#"// pop to D
@SP     //
AM=M-1  // SP--
D=M     // D = *SP
"#;
        code.to_owned()
    }

    /// for local, argument, this, that segment
    pub fn pop_to_named_segment(seg_name: &str, index: u16) -> String {
        format!(
            "/// pop to named segment\n{}{}{}\n",
            save_addr(seg_name, index),
            pop_to_d(),
            set_d_to_saved_addr()
        )
    }

    /// for pointer or temp segment
    pub fn pop_to_unnamed_segment(r_name: &str) -> String {
        format!(
            r#"/// pop to unnamed segment
{0}
@{1}
M=D
"#,
            pop_to_d(),
            r_name
        )
    }
}

/// segment[index]をR13に保存
fn save_addr(seg_name: &str, index: u16) -> String {
    format!(
        r#"// save segment index
@{0}
D=M
@{1}
D=D+A   // D = seg_name + index
@{2}
M=D
"#,
        seg_name, index, GENERIC_REG_ADDR_0
    )
}

/// R13に保存したアドレスにDを格納
fn set_d_to_saved_addr() -> String {
    format!(
        r#"// set D to segment[index]
@{0}
A=M
M=D
"#,
        GENERIC_REG_ADDR_0
    )
}
