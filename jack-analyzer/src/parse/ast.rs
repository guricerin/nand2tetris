#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ast {
    pub class: Class,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Class {
    pub name: Ident,
    pub class_var_decs: Vec<ClassVarDec>,
    pub subroutine_decs: Vec<SubRoutineDec>,
}

impl Class {
    pub fn new(
        name: Ident,
        class_var_decs: Vec<ClassVarDec>,
        subroutine_decs: Vec<SubRoutineDec>,
    ) -> Self {
        Self {
            name,
            class_var_decs,
            subroutine_decs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClassVarDec {
    pub modifier: ClassVarModifier,
    pub ty: Type,
    pub name: Ident,
    pub names: Vec<Ident>,
}

impl ClassVarDec {
    pub fn new(modifier: ClassVarModifier, ty: Type, name: Ident, names: Vec<Ident>) -> Self {
        Self {
            modifier,
            ty,
            name,
            names,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClassVarModifier {
    Static,
    Field,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Char,
    Bool,
    Class(Ident),
    Void,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubRoutineDec {
    pub modifier: SubRoutineModifier,
    pub ty: Type,
    pub name: Ident,
    pub params: ParamList,
    pub body: SubRoutineBody,
}

impl SubRoutineDec {
    pub fn new(
        modifier: SubRoutineModifier,
        ty: Type,
        name: Ident,
        params: ParamList,
        body: SubRoutineBody,
    ) -> Self {
        Self {
            modifier,
            ty,
            name,
            params,
            body,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SubRoutineModifier {
    Constructor,
    Func,
    Method,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Param(pub Type, pub Ident);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParamList(pub Option<(Param, Vec<Param>)>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubRoutineBody {
    pub vardecs: Vec<VarDec>,
    pub stmts: Stmts,
}

impl SubRoutineBody {
    pub fn new(vardecs: Vec<VarDec>, stmts: Stmts) -> Self {
        Self { vardecs, stmts }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VarDec {
    pub ty: Type,
    pub name: Ident,
    pub names: Vec<Ident>,
}

impl VarDec {
    pub fn new(ty: Type, name: Ident, names: Vec<Ident>) -> Self {
        Self { ty, name, names }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Stmts(pub Vec<Stmt>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stmt {
    Let {
        var_name: Ident,
        indexer: Option<Expr>, // array[]
        expr: Expr,
    },
    If {
        cond: Expr,
        conseq: Stmts,
        alt: Option<Stmts>,
    },
    While {
        cond: Expr,
        body: Stmts,
    },
    Do {
        subroutine_call: SubRoutineCall,
    },
    Return {
        value: Option<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr {
    pub left: Box<Term>,
    pub right: Box<Option<(BinaryOp, Term)>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Term {
    IntConst(u16),
    StringConst(String),
    Keyword(KeywordConst),
    VarName(Ident),
    Indexer(Ident, Expr), // array[]
    Call(SubRoutineCall),
    Expr(Expr),
    UnaryOp(UnaryOp, Box<Term>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SubRoutineCall {
    Func {
        name: Ident,
        exprs: ExprList,
    },
    Method {
        reciever: Ident,
        name: Ident,
        exprs: ExprList,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExprList(pub Option<(Expr, Vec<Expr>)>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Plus,  // +
    Minus, // -
    Mult,  // *
    Div,   // /
    And,   // &
    Or,    // |
    Lt,    // <
    Gt,    // >
    Eq,    // =
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Minus, // -
    Not,   // ~
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeywordConst {
    True,
    False,
    Null,
    This,
}
