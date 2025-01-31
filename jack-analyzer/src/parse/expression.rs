use super::*;

impl Parser {
    pub fn pexpr<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Expr, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let left = self.pterm(tokens)?;
        let right = {
            match self.pbinary_op(tokens) {
                Ok(op) => match self.pterm(tokens) {
                    Ok(term) => Some((op, term)),
                    Err(e) => return Err(e),
                },
                Err(_) => None,
            }
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
        match tokens.peek() {
            Some(tok) => match tok.value {
                TokenKind::Int(_) => self.pint_const(tokens),
                TokenKind::String(_) => self.pstring_const(tokens),
                TokenKind::Keyword(_) => self.pkeyword_const(tokens),
                // pterm_varnameでpidentを使ってidentを消費しているので、indexer, subcallとまとめる必要あり
                TokenKind::Ident(_) => self.pterm_varname_or_indexer_or_sub(tokens),
                TokenKind::Symbol(Symbol::LParen) => self.pterm_expr(tokens),
                TokenKind::Symbol(Symbol::Minus) => self.pterm_unary(tokens),
                TokenKind::Symbol(Symbol::Tilde) => self.pterm_unary(tokens),
                _ => Err(self.unexpected_token(tok.clone())),
            },
            _ => Err(self.eof()),
        }
    }

    fn pterm_varname_or_indexer_or_sub<Tokens>(
        &self,
        tokens: &mut Peekable<Tokens>,
    ) -> Result<Term, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let ident = self.pident(tokens)?;
        match tokens.peek() {
            Some(tok) => match tok.value {
                TokenKind::Symbol(Symbol::LSqParen) => {
                    let indexer = self.pindexer(tokens)?;
                    Ok(Term::Indexer(ident, indexer))
                }
                TokenKind::Symbol(Symbol::LParen) => {
                    let call = self.pfunc_call(tokens, ident)?;
                    Ok(Term::Call(call))
                }
                TokenKind::Symbol(Symbol::Dot) => {
                    let call = self.pmethod_call(tokens, ident)?;
                    Ok(Term::Call(call))
                }
                _ => Ok(Term::VarName(ident)),
            },
            None => Err(self.eof()),
        }
    }

    fn pterm_unary<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Term, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let op = self.punary_op(tokens)?;
        // ここで左再帰おきるかと思ったら別にそんなことはなかった
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
