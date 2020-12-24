pub static INITIALIZE: &'static str = r#"// initialize
@256 // RAM[0] (SP) = 256
D=A
@SP
M=D
"#;

// /// 使う前に、プッシュしたいデータをDに格納すること
// pub static PUSH: &'static str = r#"// push
// @SP // *SP = D
// A=M
// M=D
// @SP // SP++
// M=M+1
// "#;

/// コールする前に、プッシュしたいデータをDに格納すること
pub fn push_from_d() -> String {
    // スタックにプッシュ
    // @SPの参照先にDを入れた後、@SP自体をインクリメント
    let code = r#"// push from d
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
    let code = r#"// pop to d
@SP     //
AM=M-1  // SP--
D=M     // D = *SP
"#;
    code.to_owned()
}
