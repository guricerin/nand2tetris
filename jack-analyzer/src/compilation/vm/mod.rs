mod ast_ex;
mod symbol_table;

use crate::parse::ast::*;
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

    fn rclass_var_dec(&mut self, vardec: ClassVarDec) {
        let (ty, modifier) = (&vardec.ty, Kind::from(vardec.modifier));
        self.table.define(&vardec.name, &ty, &modifier);
        for name in vardec.names.iter() {
            self.table.define(name, &ty, &modifier);
        }
    }
    fn wsubroutine_dec(&mut self, vardec: SubRoutineDec) -> Result<(), VmWriteError> {
        self.table.start_subroutine();
        // 引数を記録
        self.rparams(&vardec.params, &vardec.modifier);
        // ローカル変数を記録
        for vardec in vardec.body.vardecs.iter() {
            self.rsub_vardec(vardec);
        }
        // function start
        self.write(format!(
            "{} {}.{} {}\n",
            &vardec.modifier,
            &self.class_name,
            &vardec.name,
            // ローカル変数の数 P.147
            &self.table.varcount(&Kind::Var)
        ));
        match &vardec.modifier {
            // slide.44 (https://drive.google.com/file/d/1DfGKr0fuJcCvlIPABNSg7fsLfFFqRLex/view)
            SubRoutineModifier::Constructor => {
                // メモリ割り当て fieldの数
                self.write(format!(
                    "    push constant {}\n",
                    &self.table.varcount(&Kind::Field)
                ));
                self.write("    call Memory.alloc 1\n");
                self.write("    pop pointer 0\n");
            }
            SubRoutineModifier::Method => {
                self.write("    push argument 0\n");
                self.write("    pop pointer 0\n");
            }
            _ => (),
        };
        self.wsubroutine_body(&vardec.body)?;
        Ok(())
    }
    /// サブルーチンの引数
    fn rparams(&mut self, params: &ParamList, modifier: &SubRoutineModifier) {
        match modifier {
            SubRoutineModifier::Method => {
                // P.265 thisを暗黙の引数として登録
                self.table.define(
                    &Ident("this".to_owned()),
                    &Type::Class(self.class_name.clone()),
                    &Kind::Arg,
                );
            }
            _ => (),
        };
        match &params.0 {
            Some((p, ps)) => {
                let kind = Kind::Arg;
                self.table.define(&p.1, &p.0, &kind);
                for p in ps.iter() {
                    self.table.define(&p.1, &p.0, &kind);
                }
            }
            None => (),
        };
    }
    /// サブルーチンのローカル変数宣言
    fn rsub_vardec(&mut self, vardec: &VarDec) {
        let kind = Kind::Var;
        self.table.define(&vardec.name, &vardec.ty, &kind);
        for name in vardec.names.iter() {
            self.table.define(name, &vardec.ty, &kind);
        }
    }
    fn wsubroutine_body(&mut self, body: &SubRoutineBody) -> Result<(), VmWriteError> {
        for stmt in body.stmts.0.iter() {
            self.wstmt(stmt)?;
        }
        Ok(())
    }
    fn wstmts(&mut self, stmts: &Stmts) -> Result<(), VmWriteError> {
        for stmt in stmts.0.iter() {
            self.wstmt(stmt)?;
        }
        Ok(())
    }
    fn wstmt(&mut self, stmt: &Stmt) -> Result<(), VmWriteError> {
        match stmt {
            Stmt::Let { .. } => self.wlet_stmt(stmt)?,
            Stmt::If { .. } => self.wif_stmt(stmt)?,
            Stmt::While { .. } => self.wwhile_stmt(stmt)?,
            Stmt::Do { .. } => {
                self.wdo_stmt(stmt)?;
                // 戻り値がvoidなのでstack topをtempセグメントにボッシュート
                self.write("    pop temp 0\n");
            }
            Stmt::Return { .. } => self.wreturn_stmt(stmt)?,
        };
        Ok(())
    }
    /// P.260
    fn wif_stmt(&mut self, stmt: &Stmt) -> Result<(), VmWriteError> {
        match stmt {
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
            // slide.73 (https://drive.google.com/file/d/1DfGKr0fuJcCvlIPABNSg7fsLfFFqRLex/view)
            // let var_name[idx] = expr
            Stmt::Let {
                var_name,
                indexer: Some(idx),
                expr,
            } => {
                let lhs = match self.get(var_name) {
                    Some(r) => r.clone(),
                    None => return Err(self.undefined_symbol(var_name)),
                };

                // slideとは違い、rhsから計算した
                // top = expr
                self.wexpr(expr)?;
                // temp[1] = expr
                self.write("    pop temp 1\n");

                // top = var_name + idx
                self.write(format!("    push {} {}\n", &lhs.kind, &lhs.id));
                self.wexpr(idx)?;
                self.write("    add\n");
                // pointer -> that segment
                self.write("    pop pointer 1\n");
                // top = expr
                self.write("    push temp 1\n");
                // *(var_name + idx) = expr
                self.write("    pop that 0\n");
            }
            _ => unreachable!(),
        }
        Ok(())
    }
    fn wdo_stmt(&mut self, stmt: &Stmt) -> Result<(), VmWriteError> {
        match stmt {
            Stmt::Do { subroutine_call } => match subroutine_call {
                SubRoutineCall::Func { .. } => self.wfunc_call(subroutine_call)?,
                SubRoutineCall::Method { .. } => self.wmethod_call(subroutine_call)?,
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
                // P.212
                let s = s.as_bytes();
                self.write(format!("    push constant {}\n", s.len()));
                self.write("    call String.new 1\n");
                for c in s.iter() {
                    self.write(format!("    push constant {}\n", c));
                    self.write("    call String.appendChar 2\n");
                }
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
                SubRoutineCall::Func { .. } => self.wfunc_call(sub)?,
                SubRoutineCall::Method { .. } => self.wmethod_call(sub)?,
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
    fn wfunc_call(&mut self, f: &SubRoutineCall) -> Result<(), VmWriteError> {
        match f {
            SubRoutineCall::Func { name, exprs } => {
                // 引数をargmumentからpushしておく
                // selfを始めにpushする
                self.write("    push pointer 0\n");
                self.wexpr_list(exprs)?; // 実引数
                self.write(format!(
                    "    call {}.{} {}\n",
                    &self.class_name,
                    name,
                    exprs.count() + 1
                ));
            }
            _ => unreachable!(),
        }
        Ok(())
    }
    fn wmethod_call(&mut self, f: &SubRoutineCall) -> Result<(), VmWriteError> {
        match f {
            SubRoutineCall::Method {
                reciever,
                name,
                exprs,
            } => {
                // 引数をpushしておく
                // recieverを先にpush
                let (ty, c) = match self.get(reciever) {
                    // インスタンス名
                    Some(r) => {
                        let obj = r.clone();
                        self.write(format!("    push {} {}\n", obj.kind, obj.id));
                        (obj.ty, 1)
                    }
                    // クラス名（型名）
                    _ => (Type::Class(reciever.clone()), 0),
                };
                // 実引数
                self.wexpr_list(exprs)?;
                // reciever.method じゃなくて Type.method
                self.write(format!("    call {}.{} {}\n", ty, name, exprs.count() + c));
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
    /// 配列への代入ではなく、値の取得を想定
    /// slide.73 (https://drive.google.com/file/d/1DfGKr0fuJcCvlIPABNSg7fsLfFFqRLex/view)
    fn windexer(&mut self, base: &Ident, idx: &Expr) -> Result<(), VmWriteError> {
        let base = match self.get(base) {
            Some(r) => r.clone(),
            None => return Err(self.undefined_symbol(base)),
        };
        // base[idx] -> base + idx
        self.write(format!("    push {} {}\n", &base.kind, &base.id));
        self.wexpr(idx)?;
        self.write("    add\n");
        // pointer -> that segment
        self.write("    pop pointer 1\n");
        // top = *(base + idx)
        self.write("    push that 0\n");
        Ok(())
    }
    /// P.263
    fn wkeyword(&mut self, k: &KeywordConst) -> Result<(), VmWriteError> {
        match k {
            KeywordConst::True => {
                // -1
                self.write("    push constant 1\n    neg\n");
            }
            KeywordConst::False | KeywordConst::Null => {
                // 0
                self.write("    push constant 0\n");
            }
            KeywordConst::This => {
                self.write("    push pointer 0\n");
            }
        };
        Ok(())
    }
}
