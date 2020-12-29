use crate::parse::ast::*;

pub fn translate(ast: &Ast) -> String {
    class(&ast.class)
}

fn class(class: &Class) -> String {
    format!(
        r#"
<class>
<keyword> class </keyword>
{}
<symbol> {} </symbol>
{}
{}
<symbol> {} </symbol>
</class>
"#,
        ident(&class.name),
        "{",
        class_var_decs(&class.class_var_decs),
        suburoutine_decs(&class.subroutine_decs),
        "}"
    )
}

fn ident(ident: &Ident) -> String {
    format!("<identifier> {} </identifier>", ident.0)
}

fn ty(ty: &Type) -> String {
    match ty {
        Type::Int => format!("<keyword> int </keyword>"),
        Type::Bool => format!("<keyword> boolean </keyword>"),
        Type::Char => format!("<keyword> char </keyword>"),
        Type::Void => format!("<keyword> void </keyword>"),
        Type::Class(id) => ident(id),
    }
}

fn class_var_decs(decs: &Vec<ClassVarDec>) -> String {
    decs.iter().map(|d| class_var_dec(d)).collect::<_>()
}

fn class_var_dec(dec: &ClassVarDec) -> String {
    todo!();
    format!(
        r#"
<classVarDec>
</classVarDec>
    "#
    )
}

fn suburoutine_decs(decs: &Vec<SubRoutineDec>) -> String {
    decs.iter().map(|dec| suburoutine_dec(&dec)).collect::<_>()
}

fn suburoutine_dec(dec: &SubRoutineDec) -> String {
    format!(
        r#"
<subroutineDec>
{}
{}
{}
<symbol> {} </symbol>
{}
<symbol> {} </symbol>
{}
</subroutineDec>
    "#,
        subroutine_modifier(&dec.modifier),
        ty(&dec.ty),
        ident(&dec.name),
        "(",
        paramlist(&dec.params),
        ")",
        subroutine_body(&dec.body),
    )
}

fn subroutine_body(body: &SubRoutineBody) -> String {
    let decs = body.vardecs.iter().map(|d| vardec(d)).collect::<String>();
    let s = stmts(&body.stmts);
    format!(
        r#"
<subroutineBody>
{}
{}
</subroutineBody>
    "#,
        decs, s
    )
}

fn vardec(dec: &VarDec) -> String {
    format!(
        r#"
<varDec>
<keyword> var </keyword>
{}
{}
{}
<symbol> ; </symbol>
</varDec>
    "#,
        ty(&dec.ty),
        ident(&dec.name),
        dec.names.iter().map(|i| ident(i)).collect::<String>(),
    )
}

fn subroutine_modifier(mo: &SubRoutineModifier) -> String {
    let a = match mo {
        SubRoutineModifier::Constructor => "constructor",
        SubRoutineModifier::Method => "method",
        SubRoutineModifier::Func => "function",
    };
    format!("<keyword> {} </keyword>", a)
}

fn paramlist(params: &ParamList) -> String {
    let a = match &params.0 {
        Some((head, tail)) => {
            let head = param(head);
            let tail = tail.iter().map(|p| param(p)).collect::<String>();
            format!("{}\n{}", head, tail)
        }
        None => "".to_owned(),
    };
    format!(
        r#"
<parameterList>
{}
</parameterList>
    "#,
        a
    )
}

fn param(param: &Param) -> String {
    format!(
        r#"
{}
{}
    "#,
        ty(&param.0),
        ident(&param.1)
    )
}

fn stmts(s: &Stmts) -> String {
    format!(
        r#"
<statements>
{}
</statements>
    "#,
        s.0.iter().map(|s| stmt(s)).collect::<String>()
    )
}

fn stmt(s: &Stmt) -> String {
    match s {
        Stmt::Let { .. } => letstmt(&s),
        Stmt::While { .. } => whilestmt(&s),
        Stmt::If { .. } => ifstmt(&s),
        Stmt::Do { .. } => dostmt(&s),
        Stmt::Return { .. } => retstmt(&s),
    }
}
fn retstmt(s: &Stmt) -> String {
    match s {
        Stmt::Return { value } => {
            let v = match value {
                Some(v) => xexpr(v),
                _ => "".to_owned(),
            };
            format!(
                r#"
<returnStatement>
<keyword> return </keyword>
{}
<symbol> ; </symbol>
</returnStatement>
            "#,
                v
            )
        }
        _ => unreachable!(),
    }
}
fn dostmt(s: &Stmt) -> String {
    match s {
        Stmt::Do { subroutine_call } => {
            format!(
                r#"
<doStatement>
<keyword> do </keyword>
{}
<symbol> ; </symbol>
</doStatement>
            "#,
                subrcall(subroutine_call)
            )
        }
        _ => unreachable!(),
    }
}
fn whilestmt(s: &Stmt) -> String {
    match s {
        Stmt::While { cond, body } => {
            format!(
                r#"
<whileStatement>
<keyword> while </keyword>
<symbol> ( </symbol>
{}
<symbol> ) </symbol>
<symbol> {} </symbol>
{}
<symbol> {} </symbol>
</whileStatement>
            "#,
                xexpr(cond),
                "{",
                stmts(body),
                "}"
            )
        }
        _ => unreachable!(),
    }
}
fn letstmt(s: &Stmt) -> String {
    let (a, b, c) = match s {
        Stmt::Let {
            var_name,
            indexer,
            expr,
        } => {
            let a = ident(var_name);
            let b = match indexer {
                Some(e) => format!(
                    r#"<symbol> [ </symbol>
                    {}
                    <symbol> ] </symbol>
                    "#,
                    xexpr(&e)
                ),
                _ => "".to_owned(),
            };
            let c = xexpr(&expr);
            (a, b, c)
        }
        _ => unreachable!(),
    };
    format!(
        r#"
<letStatement>
<keyword> let </keyword>
{}
{}
<symbol> = </symbol>
{}
<symbol> ; </symbol>
</letStatement>
    "#,
        a, b, c
    )
}
fn ifstmt(s: &Stmt) -> String {
    let (a, b, c) = match s {
        Stmt::If { cond, conseq, alt } => {
            let a = xexpr(cond);
            let b = stmts(conseq);
            let c = match alt {
                Some(e) => {
                    format!(
                        r#"
<keyword> else </keyword>
<symbol> {} </symbol>
{}
<symbol> {} </symbol>
                    "#,
                        "{",
                        stmts(e),
                        "}"
                    )
                }
                None => "".to_owned(),
            };
            (a, b, c)
        }
        _ => unreachable!(),
    };
    format!(
        r#"
<ifStatement>
<keyword> if </keyword>
<symbol> ( </symbol>
{}
<symbol> ) </symbol>
<symbol> {} </symbol>
{}
<symbol> {} </symbol>
{}
</ifStatement>
    "#,
        a, "{", b, "}", c
    )
}

fn xexpr(e: &Expr) -> String {
    let (o, a) = match &*e.right {
        Some((o, a)) => (binaryop(o), xterm(a)),
        _ => ("".to_owned(), "".to_owned()),
    };
    format!(
        r#"
{}
{}
{}
    "#,
        xterm(&*e.left),
        o,
        a
    )
}

fn binaryop(o: &BinaryOp) -> String {
    use BinaryOp::*;
    let o = match o {
        Plus => "+",
        Minus => "-",
        Mult => "*",
        Div => "/",
        And => "&amp;",
        Or => "|",
        Lt => "&lt;",
        Gt => "&gt;",
        Eq => "=",
    };
    format!("<symbol> {} </symbol>", o)
}

fn xterm(t: &Term) -> String {
    use Term::*;
    match t {
        IntConst(n) => format!("<intConstant> {} </intConstant>", n),
        StringConst(s) => format!("<stringConstant> {} </stringConstant>", s),
        Keyword(k) => keycon(k),
        VarName(s) => ident(&s),
        Indexer(i, e) => {
            format!(
                r#"
{}
<symbol> [ </symbol>
{}
<symbol> ] </symbol>
            "#,
                ident(i),
                xexpr(e)
            )
        }
        Call(c) => subrcall(c),
        Expr(e) => xexpr(e),
        UnaryOp(o, t) => {
            let a = unaryop(o);
            let b = xterm(&*t);
            format!("{}\n{}", a, b)
        }
    }
}

fn keycon(k: &KeywordConst) -> String {
    use KeywordConst::*;
    let a = match k {
        True => "true",
        False => "false",
        Null => "null",
        This => "this",
    };
    format!("<keywordConstant> {} </keywordConstant>", a)
}

fn unaryop(op: &UnaryOp) -> String {
    use UnaryOp::*;
    let a = match op {
        Minus => "-",
        Not => "~",
    };
    format!("<symbol> {} </symbol>", a)
}

fn exvec(es: &Vec<Expr>) -> String {
    let mut buf = String::new();
    for e in es.iter() {
        buf = format!("{}\n<symbol> , </symbol>\n{}", buf, xexpr(e));
    }
    buf
}

fn xexplist(e: &ExprList) -> String {
    match &e.0 {
        Some((e, es)) => {
            let e = xexpr(&e);
            let a = exvec(&es);
            format!(
                r#"
{}
{}
            "#,
                e, a
            )
        }
        _ => "".to_owned(),
    }
}

fn subrcall(s: &SubRoutineCall) -> String {
    use SubRoutineCall::*;
    match s {
        Func { name, exprs } => {
            format!(
                r#"
{}
<symbol> ( </symbol>
{}
<symbol> ) </symbol>
            "#,
                ident(name),
                xexplist(exprs)
            )
        }
        Method {
            reciever,
            name,
            exprs,
        } => {
            format!(
                r#"
{}
<symbol> . </symbol>
{}
<symbol> ( </symbol>
{}
<symbol> ) </symbol>
            "#,
                ident(reciever),
                ident(name),
                xexplist(exprs)
            )
        }
    }
}
