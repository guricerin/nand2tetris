use super::vm_bool::VmBool;

pub use flow::*;
pub use func::*;
pub use jump::*;
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
static GENERIC_REG_ADDR_1: &'static str = "R14";
// static GENERIC_REG_ADDR_2: &'static str = "R15";

pub mod operate {
    /// SP-- の後に *(SP-1) = expr
    pub fn binary_op(expr: &str) -> String {
        // スタックからrightをポップ
        // スタックの頂点(left)を left op right に書き換える
        // A=A-1 は、RAM[@SP]を-1するわけではない
        format!(
            r#"/// binary op
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
            r#"/// unary op
@SP     // ptr = SP
A=M-1   // ptr = ptr - 1
M={}M   // *(ptr) = op *(ptr)
"#,
            op
        )
    }
}

pub mod jump {
    use super::*;

    pub fn jump(jmp: &str, true_label: &str, false_label: &str, break_label: &str) -> String {
        format!(
            r#"/// jump
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

pub mod flow {
    /// ラベルが一意になるように加工
    fn edit_label(filename: &str, funcname: &str, org_label: &str) -> String {
        format!("{}.{}.{}", filename, funcname, org_label)
    }
    pub fn label(filename: &str, funcname: &str, org_label: &str) -> String {
        format!(
            r#"/// label ({0})
({1})
"#,
            org_label,
            edit_label(filename, funcname, org_label)
        )
    }
    pub fn goto(filename: &str, funcname: &str, org_label: &str) -> String {
        format!(
            r#"/// goto ({0})
@{1}
0;JMP
"#,
            org_label,
            edit_label(filename, funcname, org_label)
        )
    }
    pub fn ifgoto(filename: &str, funcname: &str, org_label: &str) -> String {
        use super::*;
        format!(
            r#"/// if-goto ({0})
{1}
@{2}
D;JNE   // D != 0 -> jump
"#,
            org_label,
            pop_to_d(),
            edit_label(filename, funcname, org_label)
        )
    }
}

pub mod func {
    use super::*;

    fn func_start_label(filename: &str, funcname: &str) -> String {
        format!("{}", funcname)
    }
    pub fn func(filename: &str, funcname: &str, paramc: u16) -> String {
        let push_0 = format!(
            r#"// `push 0` for LCL
@0
D=A
{0}
"#,
            push_from_d()
        );
        format!(
            r#"/// function {0} {1}
({2})   // function start arddress
{3}
"#,
            funcname,
            paramc,
            func_start_label(filename, funcname),
            push_0.repeat(paramc as usize) // funcnameにとってのLCLを初期化
        )
    }

    fn return_address_label(filename: &str, funcname: &str, id: u64) -> String {
        // format!(r#"_RETURN_TO_{0}.{1}:{2}_"#, filename, funcname, id)
        format!(r#"_RETURN_TO_{0}:{1}_"#, funcname, id)
    }
    /// 呼び出し側のtargetを復元
    fn restore(target: &str, saving_frame_addr: &str, offset: u16) -> String {
        format!(
            r#"// restore
@{0}
D=A
@{1}
A=M-D   // ptr = FRAME - offset
D=M     // D = *(FRAME - offset)
@{2}
M=D     // TARGET = *(FRAME - offset)
"#,
            offset, saving_frame_addr, target
        )
    }
    pub fn f_return() -> String {
        let saving_frame_addr = GENERIC_REG_ADDR_0;
        let saving_return_addr = GENERIC_REG_ADDR_1;
        format!(
            r#"/// return
@LCL
D=M     // D = FRAME
@{0}
M=D     // save FRAME
@5
A=D-A   // RAM[(FRAME-5)]
D=M     // D = *(FRAME - 5) <- RET
@{1}
M=D     // save RET
{2}
@ARG    // save return value for caller
A=M
M=D     // *ARG = pop()
@ARG
D=M+1   // D = ARG + 1
@SP
M=D     // SP = ARG + 1
{3}
{4}
{5}
{6}
@{1}     // go to RET(caller)
A=M
0;JMP
"#,
            saving_frame_addr,
            saving_return_addr,
            pop_to_d(),
            restore("THAT", saving_frame_addr, 1),
            restore("THIS", saving_frame_addr, 2),
            restore("ARG", saving_frame_addr, 3),
            restore("LCL", saving_frame_addr, 4),
        )
    }

    fn push_addr(label: &str) -> String {
        format!(
            r#"// save addr to D
@{0}
D=M
{1}
"#,
            label,
            push_from_d()
        )
    }
    // 現在のSPをこの関数にとってのLCLとしてあつかうので、どっかに保持
    pub fn call(filename: &str, caller: &str, callee: &str, argc: u16, id: u64) -> String {
        let return_addr = return_address_label(filename, caller, id);
        let push_return_addr = format!(
            r#"// push return-addr
@{0}
D=A
{1}
"#,
            return_addr,
            push_from_d()
        );
        let move_arg = format!(
            r#"// ARG = SP - argc - 5
@{0}
D=A
@SP
D=M-D
@ARG
M=D
"#,
            argc + 5
        );
        let move_lcl = format!(
            r#"// LCL = SP
@SP
D=M
@LCL
M=D
"#
        );
        let invoke_callee = format!(
            r#"
@{0}    // invoke callee
0;JMP
({1})   // return to caller
"#,
            func_start_label(filename, callee),
            return_addr
        );
        format!(
            r#"/// call {0} {1}
{2}
{3}
{4}
{5}
{6}
{7}
{8}
{9}
"#,
            callee,
            argc,
            push_return_addr,
            push_addr("LCL"),
            push_addr("ARG"),
            push_addr("THIS"),
            push_addr("THAT"),
            move_arg,
            move_lcl,
            invoke_callee,
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
D=D+A   // D = seg_base_addr + index
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
