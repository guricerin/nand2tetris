mod ast_ex;
mod symbol_table;

use crate::parse::ast::*;
use std::fmt;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use symbol_table::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VmWriteError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}\nundefined symbol: {1}")]
    UndefinedSymbol(PathBuf, Ident),
}

pub struct VmWriter {
    out_path: PathBuf,
    writer: BufWriter<File>,
    table: SymbolTable,
    class_name: Ident,
    goto_label_id: u64,
}

enum StackOp {
    Push,
    Pop,
}

impl fmt::Display for StackOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StackOp::Push => write!(f, "push"),
            StackOp::Pop => write!(f, "pop"),
        }
    }
}

impl VmWriter {
    fn undefined_symbol(&self, symbol: &Ident) -> VmWriteError {
        VmWriteError::UndefinedSymbol(self.out_path.clone(), symbol.clone())
    }
}

impl VmWriter {
    fn new(out_path: PathBuf, class_name: Ident) -> Result<Self, VmWriteError> {
        let writer = BufWriter::new(File::create(out_path.clone())?);
        Ok(Self {
            out_path,
            writer,
            table: SymbolTable::new(),
            class_name,
            goto_label_id: 0,
        })
    }
    pub fn run(out_path: PathBuf, ast: Ast) -> Result<(), VmWriteError> {
        let mut vmwriter = VmWriter::new(out_path, ast.class.name.clone())?;
        vmwriter.travarse(ast.class)?;
        vmwriter.writer.flush()?;
        Ok(())
    }
    fn travarse(&mut self, class: Class) -> Result<(), VmWriteError> {
        for vardec in class.class_var_decs.into_iter() {
            self.rclass_var_dec(vardec);
        }
        for vardec in class.subroutine_decs.into_iter() {
            self.wsubroutine_dec(vardec)?;
        }
        Ok(())
    }

    fn write<S: Into<String>>(&mut self, s: S) {
        let _ = self.writer.write(s.into().as_bytes());
    }
    fn get(&self, s: &Ident) -> Option<&Record> {
        self.table.get(s)
    }
    fn dbg_recode(&self, i: &Ident) {
        let r = self.table.get(i);
        println!("{:?}: {:?}", i, r)
    }
    // フィールドのn番目の値に引数の1番目の値をセットする例：``push argument 1  pop this n``
    fn access_field(&mut self) {
        self.write("pop pointer 0\n");
    }

    fn rclass_var_dec(&mut self, vardec: ClassVarDec) {
        let (ty, modifier) = (&vardec.ty, Kind::from(vardec.modifier));
        self.table.define(&vardec.name, &ty, &modifier);
        self.dbg_recode(&vardec.name);
        for name in vardec.names.iter() {
            self.table.define(name, &ty, &modifier);
            self.dbg_recode(&name);
        }
    }
    fn wsubroutine_dec(&mut self, vardec: SubRoutineDec) -> Result<(), VmWriteError> {
        self.table.start_subroutine();
        self.rparams(&vardec.params);

        // function start
        self.write(format!(
            "{} {}.{} {}\n",
            &vardec.modifier,
            &self.class_name,
            &vardec.name,
            &vardec.params.count()
        ));
        self.wsubroutine_body(&vardec.body)?;
        Ok(())
    }
    /// サブルーチンの引数
    fn rparams(&mut self, params: &ParamList) {
        // P.265 thisを暗黙の引数として登録
        self.table.define(
            &Ident("this".to_owned()),
            &Type::Class(self.class_name.clone()),
            &Kind::Arg,
        );
        self.dbg_recode(&Ident("this".to_owned()));
        match &params.0 {
            Some((p, ps)) => {
                let kind = Kind::Arg;
                self.table.define(&p.1, &p.0, &kind);
                self.dbg_recode(&p.1);
                for p in ps.iter() {
                    self.table.define(&p.1, &p.0, &kind);
                    self.dbg_recode(&p.1);
                }
            }
            None => (),
        };
    }
    fn wsubroutine_body(&mut self, body: &SubRoutineBody) -> Result<(), VmWriteError> {
        for vardec in body.vardecs.iter() {
            self.rsub_vardec(vardec);
        }
        for stmt in body.stmts.0.iter() {
            self.wstmt(stmt)?;
        }
        Ok(())
    }
    /// サブルーチンのローカル変数宣言
    fn rsub_vardec(&mut self, vardec: &VarDec) {
        let kind = Kind::Var; // たぶんArgじゃない。Varは関数内のローカル変数だから。
        self.table.define(&vardec.name, &vardec.ty, &kind);
        self.dbg_recode(&vardec.name);
        for name in vardec.names.iter() {
            self.table.define(name, &vardec.ty, &kind);
            self.dbg_recode(name);
        }
    }
    fn wstmts(&mut self, stmts: &Stmts) -> Result<(), VmWriteError> {
        for stmt in stmts.0.iter() {
            self.wstmt(stmt)?;
        }
        Ok(())
    }
    fn wstmt(&mut self, stmt: &Stmt) -> Result<(), VmWriteError> {
        // ここからleft, right, nodeの順でコード化する
        match stmt {
            Stmt::Let { .. } => self.wlet_stmt(stmt)?,
            // if goto if-goto label
            Stmt::If { .. } => self.wif_stmt(stmt)?,
            // if goto if-goto label
            Stmt::While { .. } => self.wwhile_stmt(stmt)?,
            // call
            Stmt::Do { .. } => self.wdo_stmt(stmt)?,
            Stmt::Return { .. } => self.wreturn_stmt(stmt)?,
        };
        Ok(())
    }
    /// P.260
    fn wif_stmt(&mut self, stmt: &Stmt) -> Result<(), VmWriteError> {
        match stmt {
            // if goto if-goto label
            Stmt::If { cond, conseq, alt } => {
                let else_label = format!("IF_ELSE_{}", self.goto_label_id);
                let break_label = format!("IF_BREAK_{}", self.goto_label_id);
                self.goto_label_id += 1;

                self.wexpr(cond)?;
                self.write(format!("    not\n    if-goto {}\n", &else_label));
                self.wstmts(conseq)?;
                self.write(format!("    goto {}\n", &break_label));
                self.write(format!("label {}\n", &else_label));
                match alt {
                    Some(alt) => self.wstmts(alt)?,
                    None => (),
                }
                self.write(format!("label {}\n", &break_label));
            }
            _ => unreachable!(),
        };
        Ok(())
    }
    /// P.260
    fn wwhile_stmt(&mut self, stmt: &Stmt) -> Result<(), VmWriteError> {
        match stmt {
            // if goto if-goto label
            Stmt::While { cond, body } => {
                let loop_label = format!("WHILE_LOOP_{}", self.goto_label_id);
                let break_label = format!("WHILE_BREAK_{}", self.goto_label_id);
                self.goto_label_id += 1;

                self.write(format!("label {}\n", &loop_label));
                // stack top: true => -1, false => 0
                self.wexpr(cond)?;
                // if-goto: stack topが0以外ならjmp
                self.write(format!("    not\n    if-goto {}\n", &break_label));
                self.wstmts(body)?;
                self.write(format!("    goto {}\n", &loop_label));
                self.write(format!("label {}\n", &break_label));
            }
            _ => unreachable!(),
        };
        Ok(())
    }
    fn wlet_stmt(&mut self, stmt: &Stmt) -> Result<(), VmWriteError> {
        match stmt {
            Stmt::Let {
                var_name,
                indexer: None,
                expr,
            } => {
                let lhs = match self.get(var_name) {
                    // 参照のままだと下の方で``self as mutable borrow``などと切れられる
                    Some(r) => r.clone(),
                    // let文のlhsは定義されているべき?
                    None => return Err(self.undefined_symbol(var_name)),
                };

                // rhsの結果がpushされるはず
                self.wexpr(expr)?;
                // 結果をlhsに代入
                self.write(format!("    pop {} {}\n", lhs.kind, lhs.id));
            }
            Stmt::Let {
                var_name,
                indexer: Some(idx),
                expr,
            } => {
                // P.235
                self.wexpr(expr)?;
                self.windexer(var_name, idx)?;
                self.write("    pop that 0\n");
            }
            _ => unreachable!(),
        }
        Ok(())
    }
    fn wdo_stmt(&mut self, stmt: &Stmt) -> Result<(), VmWriteError> {
        match stmt {
            Stmt::Do { subroutine_call } => match subroutine_call {
                SubRoutineCall::Func { .. } => self.wfunc_call(subroutine_call, true)?,
                SubRoutineCall::Method { .. } => self.wmethod_call(subroutine_call, true)?,
            },
            _ => unreachable!(),
        }
        Ok(())
    }
    fn wreturn_stmt(&mut self, stmt: &Stmt) -> Result<(), VmWriteError> {
        match stmt {
            Stmt::Return { value: Some(expr) } => {
                self.wexpr(expr)?;
                self.write("    return\n");
            }
            Stmt::Return { value: None } => {
                self.write("    push constant 0\n    return\n");
            }
            _ => unreachable!(),
        }
        Ok(())
    }
    fn wexpr(&mut self, expr: &Expr) -> Result<(), VmWriteError> {
        self.wterm(&*expr.left)?;
        match &*expr.right {
            Some((op, right)) => {
                self.wterm(right)?;
                self.write(format!("    {}\n", op));
            }
            _ => (),
        };
        Ok(())
    }
    fn wterm(&mut self, term: &Term) -> Result<(), VmWriteError> {
        match term {
            Term::IntConst(n) => {
                self.write(format!("    push constant {}\n", n));
            }
            Term::StringConst(s) => {
                // todo: わからん
                self.write("    call String.new 0\n");
            }
            Term::Keyword(k) => {
                self.wkeyword(k)?;
            }
            Term::VarName(v) => match self.get(v) {
                Some(r) => {
                    let r = r.clone();
                    self.write(format!("    push {} {}\n", r.kind, r.id));
                }
                None => {
                    return Err(self.undefined_symbol(v));
                }
            },
            Term::Indexer(s, expr) => {
                self.windexer(s, expr)?;
            }
            Term::Call(sub) => match sub {
                SubRoutineCall::Func { .. } => self.wfunc_call(sub, false)?,
                SubRoutineCall::Method { .. } => self.wmethod_call(sub, false)?,
            },
            Term::Expr(expr) => {
                self.wexpr(expr)?;
            }
            Term::UnaryOp(op, term) => {
                self.wterm(&*term)?;
                self.write(format!("    {}\n", op));
            }
        }
        Ok(())
    }
    /// 戻り値がvoidのfunc or method はcallしたあとpopしなければならないが、どうやってvoidかどうかを判定する？
    /// => do文で使われたかで判定する
    fn wfunc_call(&mut self, f: &SubRoutineCall, is_void: bool) -> Result<(), VmWriteError> {
        match f {
            SubRoutineCall::Func { name, exprs } => {
                // 引数をargmumentからpushしておく
                // selfを始めにpushする
                self.write("    pop pointer 0\n");
                self.write("    push this 0\n");

                self.wexpr_list(exprs)?; // 実引数
                self.write(format!("    call {} {}\n", name, exprs.count() + 1));
                if is_void {
                    // tempセグメントをゴミ箱として扱う
                    self.write("    pop temp 0\n");
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }
    fn wmethod_call(&mut self, f: &SubRoutineCall, is_void: bool) -> Result<(), VmWriteError> {
        match f {
            SubRoutineCall::Method {
                reciever,
                name,
                exprs,
            } => {
                // 引数をpushしておく
                self.wexpr_list(exprs)?; // 実引数
                self.write(format!(
                    "    call {}.{} {}\n",
                    reciever,
                    name,
                    exprs.count()
                ));
                if is_void {
                    // tempセグメントをゴミ箱として扱う
                    self.write("    pop temp 0\n");
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }
    fn wexpr_list(&mut self, exprs: &ExprList) -> Result<(), VmWriteError> {
        match &exprs.0 {
            Some((e, es)) => {
                self.wexpr(e)?;
                for e in es.iter() {
                    self.wexpr(e)?;
                }
            }
            None => (),
        }
        Ok(())
    }
    /// todo: thatセグメントとか誰も使ってないからなおせ
    fn windexer(&mut self, base: &Ident, idx: &Expr) -> Result<(), VmWriteError> {
        let base = match self.get(base) {
            Some(r) => r.clone(),
            None => return Err(self.undefined_symbol(base)),
        };
        // base[idx] -> base + idx
        self.wexpr(idx)?;
        self.write(format!("    push {} {}\n", &base.kind, &base.id));
        self.write("    add\n");
        // pointer -> that segment
        self.write("    pop pointer 1\n");
        Ok(())
    }
    fn wkeyword(&mut self, k: &KeywordConst) -> Result<(), VmWriteError> {
        match k {
            KeywordConst::True => {
                // -1
                self.write("    push constant 1\nneg\n");
            }
            KeywordConst::False | KeywordConst::Null => {
                // 0
                self.write("    push constant 0\n");
            }
            KeywordConst::This => {
                // todo: selfの指し方がわからん
                self.write("    pop pointer 0\n");
                self.write("    push this 0\n");
            }
        };
        Ok(())
    }
}
