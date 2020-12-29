use super::*;

impl Parser {
    pub fn pexpr<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Expr, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let left = self.pterm(tokens)?;
        let right = match self.pbinary_op(tokens) {
            Ok(op) => match self.pterm(tokens) {
                Ok(term) => Some((op, term)),
                Err(e) => return Err(e),
            },
            Err(_) => None,
        };
        let left = Box::new(left);
        let right = Box::new(right);
        Ok(Expr { left, right })
    }

    pub fn pindexer<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Expr, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        // [
        let _ = self.skip_symbol(tokens, Symbol::LSqParen)?;
        let expr = self.pexpr(tokens)?;
        // ]
        let _ = self.skip_symbol(tokens, Symbol::RSqParen)?;
        Ok(expr)
    }

    fn pterm<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Term, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        // クロージャだとmoveがおきてコンパイラに切れられるので、各自専用のメソッドを定義した
        // let pterm_expr = |tokens| -> Result<Term, ParseError> {
        //     let _ = self.skip_symbol(tokens, Symbol::LParen)?;
        //     let sub = self.psubroutine_call(tokens)?;
        //     let _ = self.skip_symbol(tokens, Symbol::RParen)?;
        //     Ok(Term::Call(sub))
        // };
        self.pint_const(tokens)
            .or(self.pstring_const(tokens))
            .or(self.pkeyword_const(tokens))
            .or(self.pterm_varname(tokens))
            .or(self.pterm_indexer(tokens))
            .or(self.pterm_sub(tokens))
            .or(self.pterm_expr(tokens))
            .or(self.pterm_unary(tokens))
    }

    fn pterm_sub<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Term, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let sub = self.psubroutine_call(tokens)?;
        Ok(Term::Call(sub))
    }

    fn pterm_unary<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Term, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let op = self.punary_op(tokens)?;
        // todo: ここで左再帰おきるかも
        let term = self.pterm(tokens)?;
        Ok(Term::UnaryOp(op, Box::new(term)))
    }

    fn pterm_expr<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Term, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let _ = self.skip_symbol(tokens, Symbol::LParen)?;
        let expr = self.pexpr(tokens)?;
        let _ = self.skip_symbol(tokens, Symbol::RParen)?;
        Ok(Term::Expr(expr))
    }

    fn pint_const<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Term, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let int = tokens.peek().ok_or(self.eof()).and_then(|tok| match tok {
            Token {
                value: TokenKind::Int(n),
                ..
            } => Ok(Term::IntConst(*n)),
            _ => Err(self.unexpected_token(tok.clone())),
        })?;
        tokens.next();
        Ok(int)
    }

    fn pstring_const<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Term, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let int = tokens.peek().ok_or(self.eof()).and_then(|tok| match tok {
            Token {
                value: TokenKind::String(s),
                ..
            } => Ok(Term::StringConst(s.to_owned())),
            _ => Err(self.unexpected_token(tok.clone())),
        })?;
        tokens.next();
        Ok(int)
    }

    fn pkeyword_const<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Term, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let key = tokens.peek().ok_or(self.eof()).and_then(|tok| match tok {
            Token {
                value: TokenKind::Keyword(k),
                ..
            } => match k {
                Keyword::True => Ok(KeywordConst::True),
                Keyword::False => Ok(KeywordConst::False),
                Keyword::Null => Ok(KeywordConst::Null),
                Keyword::This => Ok(KeywordConst::This),
                _ => Err(self.unexpected_token(tok.clone())),
            },
            _ => Err(self.unexpected_token(tok.clone())),
        })?;
        tokens.next();
        Ok(Term::Keyword(key))
    }

    fn pterm_varname<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Term, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let var_name = self.pident(tokens)?;
        Ok(Term::VarName(var_name))
    }

    fn pterm_indexer<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Term, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let var_name = self.pident(tokens)?;
        let indexer = self.pindexer(tokens)?;
        Ok(Term::Indexer(var_name, indexer))
    }

    fn pexpr_list<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<ExprList, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let head = self.pexpr(tokens).ok();
        match head {
            None => return Ok(ExprList(None)),
            Some(_) => (),
        };

        // (, expr)*
        let mut tail = vec![];
        loop {
            match self.skip_symbol(tokens, Symbol::Comma) {
                Ok(_) => match self.pexpr(tokens) {
                    Ok(expr) => {
                        tail.push(expr);
                    }
                    Err(e) => return Err(e),
                },
                Err(_) => break,
            }
        }

        Ok(ExprList(Some((head.unwrap(), tail))))
    }

    // fun(arg) | objext.method(arg)
    pub fn psubroutine_call<Tokens>(
        &self,
        tokens: &mut Peekable<Tokens>,
    ) -> Result<SubRoutineCall, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let ident = self.pident(tokens)?;
        match tokens.peek() {
            Some(tok) => match tok {
                Token {
                    value: TokenKind::Symbol(s),
                    ..
                } => match s {
                    Symbol::LParen => self.pfunc_call(tokens, ident),
                    Symbol::Dot => self.pmethod_call(tokens, ident),
                    _ => Err(self.unexpected_token(tok.clone())),
                },
                _ => Err(self.unexpected_token(tok.clone())),
            },
            None => Err(self.eof()),
        }
    }

    fn pfunc_call<Tokens>(
        &self,
        tokens: &mut Peekable<Tokens>,
        name: Ident,
    ) -> Result<SubRoutineCall, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        tokens.next();
        // name ( expr )
        let _ = self.skip_symbol(tokens, Symbol::LParen)?;
        let exprs = self.pexpr_list(tokens)?;
        let _ = self.skip_symbol(tokens, Symbol::RParen)?;
        Ok(SubRoutineCall::Func { name, exprs })
    }

    fn pmethod_call<Tokens>(
        &self,
        tokens: &mut Peekable<Tokens>,
        reciever: Ident,
    ) -> Result<SubRoutineCall, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        tokens.next();
        // reciever.method(expr)
        let name = self.pident(tokens)?;
        let _ = self.skip_symbol(tokens, Symbol::LParen)?;
        let exprs = self.pexpr_list(tokens)?;
        let _ = self.skip_symbol(tokens, Symbol::RParen)?;
        Ok(SubRoutineCall::Method {
            reciever,
            name,
            exprs,
        })
    }

    fn pbinary_op<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<BinaryOp, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let op = tokens.peek().ok_or(self.eof()).and_then(|tok| match tok {
            Token {
                value: TokenKind::Symbol(s),
                ..
            } => match s {
                Symbol::Plus => Ok(BinaryOp::Plus),
                Symbol::Minus => Ok(BinaryOp::Minus),
                Symbol::Asterisk => Ok(BinaryOp::Mult),
                Symbol::Slash => Ok(BinaryOp::Div),
                Symbol::And => Ok(BinaryOp::And),
                Symbol::Or => Ok(BinaryOp::Or),
                Symbol::Lt => Ok(BinaryOp::Lt),
                Symbol::Gt => Ok(BinaryOp::Gt),
                Symbol::Eq => Ok(BinaryOp::Eq),
                _ => Err(self.unexpected_token(tok.clone())),
            },
            _ => Err(self.unexpected_token(tok.clone())),
        })?;
        tokens.next();
        Ok(op)
    }

    fn punary_op<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<UnaryOp, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let op = tokens.peek().ok_or(self.eof()).and_then(|tok| match tok {
            Token {
                value: TokenKind::Symbol(s),
                ..
            } => match s {
                Symbol::Minus => Ok(UnaryOp::Minus),
                Symbol::Tilde => Ok(UnaryOp::Not),
                _ => Err(self.unexpected_token(tok.clone())),
            },
            _ => Err(self.unexpected_token(tok.clone())),
        })?;
        tokens.next();
        Ok(op)
    }
}
