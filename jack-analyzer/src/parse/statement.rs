use super::*;

impl Parser {
    pub fn pstmts<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Stmts, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let mut stmts = vec![];
        loop {
            match self.pstmt(tokens) {
                Ok(stmt) => stmts.push(stmt),
                Err(_) => break,
            }
        }
        Ok(Stmts(stmts))
    }

    fn pstmt<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Stmt, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        // ok_or().and_then()だとその中でtokensを他のメソッドにわたせない
        match tokens.peek() {
            Some(Token {
                value: TokenKind::Keyword(Keyword::Let),
                ..
            }) => {
                tokens.next();
                self.plet(tokens)
            }
            Some(Token {
                value: TokenKind::Keyword(Keyword::If),
                ..
            }) => {
                tokens.next();
                self.pif(tokens)
            }
            Some(Token {
                value: TokenKind::Keyword(Keyword::While),
                ..
            }) => {
                tokens.next();
                self.pwhile(tokens)
            }
            Some(Token {
                value: TokenKind::Keyword(Keyword::Do),
                ..
            }) => {
                tokens.next();
                self.pdo(tokens)
            }
            Some(Token {
                value: TokenKind::Keyword(Keyword::Return),
                ..
            }) => {
                tokens.next();
                self.preturn(tokens)
            }
            Some(tok) => Err(self.unexpected_token(tok.clone())),
            None => Err(self.eof()),
        }
    }

    fn plet<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Stmt, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let var_name = self.pident(tokens)?;
        let indexer = self.pindexer(tokens).ok();
        // =
        let _ = self.skip_symbol(tokens, Symbol::Eq)?;
        let expr = self.pexpr(tokens)?;
        // ;
        let _ = self.skip_symbol(tokens, Symbol::SemiColon)?;

        let res = Stmt::Let {
            var_name,
            indexer,
            expr,
        };
        Ok(res)
    }

    fn pif<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Stmt, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        // ( cond )
        let _ = self.skip_symbol(tokens, Symbol::LParen)?;
        let cond = self.pexpr(tokens)?;
        let _ = self.skip_symbol(tokens, Symbol::RParen)?;
        // { conseq }
        let _ = self.skip_symbol(tokens, Symbol::LCurlyParen)?;
        let conseq = self.pstmts(tokens)?;
        let _ = self.skip_symbol(tokens, Symbol::RCurlyParen)?;

        let mut pelse = || -> Result<Stmts, ParseError> {
            tokens.peek().ok_or(self.eof()).and_then(|tok| match tok {
                Token {
                    value: TokenKind::Keyword(Keyword::Else),
                    ..
                } => Ok(()),
                _ => Err(self.unexpected_token(tok.clone())),
            })?;
            tokens.next();
            let _ = self.skip_symbol(tokens, Symbol::LCurlyParen)?;
            let alt = self.pstmts(tokens)?;
            let _ = self.skip_symbol(tokens, Symbol::RCurlyParen)?;
            Ok(alt)
        };
        let alt = pelse().ok();

        Ok(Stmt::If { cond, conseq, alt })
    }

    fn pwhile<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Stmt, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        // ( cond )
        let _ = self.skip_symbol(tokens, Symbol::LParen)?;
        let cond = self.pexpr(tokens)?;
        let _ = self.skip_symbol(tokens, Symbol::RParen)?;
        // { body }
        let _ = self.skip_symbol(tokens, Symbol::LCurlyParen)?;
        let body = self.pstmts(tokens)?;
        let _ = self.skip_symbol(tokens, Symbol::RCurlyParen)?;
        Ok(Stmt::While { cond, body })
    }

    fn pdo<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Stmt, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let subroutine_call = self.psubroutine_call(tokens)?;
        // ;
        let _ = self.skip_symbol(tokens, Symbol::SemiColon)?;
        Ok(Stmt::Do { subroutine_call })
    }

    fn preturn<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Stmt, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let value = self.pexpr(tokens).ok();
        // ;
        let _ = self.skip_symbol(tokens, Symbol::SemiColon)?;
        Ok(Stmt::Return { value })
    }
}
