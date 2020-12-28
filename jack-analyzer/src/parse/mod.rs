mod ast;
mod expression;
mod statement;

use crate::lex::token::*;
use ast::*;
use std::iter::Peekable;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("{0}\n unexpected token\n{1}")]
    UnexpectedToken(PathBuf, Token),
    #[error("{0} unexpected eof")]
    Eof(PathBuf),
    #[error("{0}\n redundant token\n{1}")]
    RedundantToken(PathBuf, Token),
}

pub struct Parser {
    file_path: PathBuf,
}

impl Parser {
    pub fn new(file_path: &PathBuf) -> Self {
        Self {
            file_path: file_path.to_owned(),
        }
    }

    pub fn run(&self, tokens: Vec<Token>) -> Result<Ast, ParseError> {
        let mut tokens = tokens.into_iter().peekable();
        self.ast(&mut tokens)
    }

    fn unexpected_token(&self, tok: Token) -> ParseError {
        ParseError::UnexpectedToken(self.file_path.to_owned(), tok)
    }
    fn eof(&self) -> ParseError {
        ParseError::Eof(self.file_path.to_owned())
    }
    fn redundant_token(&self, tok: Token) -> ParseError {
        ParseError::RedundantToken(self.file_path.to_owned(), tok)
    }

    fn ast<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Ast, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let class = self.class(tokens)?;
        match tokens.next() {
            None => Ok(Ast { class }),
            // 1ファイル1クラス想定
            Some(t) => Err(self.redundant_token(t)),
        }
    }

    fn class<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Class, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        match tokens.peek() {
            Some(Token {
                value: TokenKind::Keyword(Keyword::Class),
                ..
            }) => {
                tokens.next();
                // class name
                let name = self.ident(tokens)?;
                // {
                let _ = self.skip_symbol(tokens, Symbol::LCurlyParen)?;
                // vardec*
                let mut var_decs = vec![];
                loop {
                    match self.class_var_dec(tokens) {
                        Ok(decs) => {
                            var_decs.push(decs);
                        }
                        Err(_) => break,
                    }
                }
                // subroutinedec*
                let mut subroutine_decs = vec![];
                loop {
                    match self.subroutine_dec(tokens) {
                        Ok(decs) => {
                            subroutine_decs.push(decs);
                        }
                        Err(_) => break,
                    }
                }
                // }
                let _ = self.skip_symbol(tokens, Symbol::RCurlyParen)?;
                Ok(Class::new(name, var_decs, subroutine_decs))
            }
            Some(t) => return Err(self.unexpected_token(t.clone())),
            None => return Err(self.eof()),
        }
    }

    fn ident<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Ident, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let ident = tokens.peek().ok_or(self.eof()).and_then(|tok| match tok {
            Token {
                value: TokenKind::Ident(name),
                ..
            } => Ok(Ident(name.clone())),
            t => Err(self.unexpected_token(t.clone())),
        })?;
        tokens.next();
        Ok(ident)
    }

    fn skip_symbol<Tokens>(
        &self,
        tokens: &mut Peekable<Tokens>,
        expect: Symbol,
    ) -> Result<(), ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let _ = tokens.peek().ok_or(self.eof()).and_then(|tok| match tok {
            Token {
                value: TokenKind::Symbol(s),
                ..
            } if &expect == s => Ok(()),
            tok => Err(self.unexpected_token(tok.clone())),
        })?;
        tokens.next();
        Ok(())
    }

    fn class_var_dec<Tokens>(
        &self,
        tokens: &mut Peekable<Tokens>,
    ) -> Result<ClassVarDec, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let modifier = tokens.peek().ok_or(self.eof()).and_then(|tok| match tok {
            Token {
                value: TokenKind::Keyword(k),
                ..
            } => match k {
                Keyword::Field => Ok(ClassVarModifier::Field),
                Keyword::Static => Ok(ClassVarModifier::Static),
                _ => Err(self.unexpected_token(tok.clone())),
            },
            _ => Err(self.unexpected_token(tok.clone())),
        })?;
        tokens.next();

        let ty = self.ty(tokens)?;
        let name = self.ident(tokens)?;
        // , と identをとれるだけとる
        let mut names = vec![];
        loop {
            match self.skip_symbol(tokens, Symbol::Comma) {
                Ok(()) => match self.ident(tokens) {
                    Ok(ident) => {
                        names.push(ident);
                    }
                    // , の次は varnameであるべき
                    Err(e) => return Err(e),
                },
                Err(_) => break,
            }
        }
        let res = ClassVarDec::new(modifier, ty, name, names);
        tokens.next();
        Ok(res)
    }

    fn ty<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Type, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let ty = tokens.peek().ok_or(self.eof()).and_then(|tok| match tok {
            Token {
                value: TokenKind::Keyword(k),
                ..
            } => match k {
                Keyword::Int => Ok(Type::Int),
                Keyword::Bool => Ok(Type::Bool),
                Keyword::Char => Ok(Type::Char),
                _ => Err(self.unexpected_token(tok.clone())),
            },
            // todo: 自信ない
            Token {
                value: TokenKind::Ident(s),
                ..
            } => Ok(Type::Class(Ident(s.clone()))),
            _ => Err(self.unexpected_token(tok.clone())),
        })?;
        tokens.next();
        Ok(ty)
    }

    fn subroutine_dec<Tokens>(
        &self,
        tokens: &mut Peekable<Tokens>,
    ) -> Result<SubRoutineDec, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let modifier = tokens.peek().ok_or(self.eof()).and_then(|tok| match tok {
            Token {
                value: TokenKind::Keyword(k),
                ..
            } => match k {
                Keyword::Constructor => Ok(SubRoutineModifier::Constructor),
                Keyword::Func => Ok(SubRoutineModifier::Func),
                Keyword::Method => Ok(SubRoutineModifier::Method),
                _ => Err(self.unexpected_token(tok.clone())),
            },
            _ => Err(self.unexpected_token(tok.clone())),
        })?;
        tokens.next();

        let ty = tokens.peek().ok_or(self.eof()).and_then(|tok| match tok {
            Token {
                value: TokenKind::Keyword(k),
                ..
            } => match k {
                Keyword::Void => Ok(Type::Void),
                Keyword::Int => Ok(Type::Int),
                Keyword::Char => Ok(Type::Char),
                Keyword::Bool => Ok(Type::Bool),
                _ => Err(self.unexpected_token(tok.clone())),
            },
            Token {
                value: TokenKind::Ident(s),
                ..
            } => Ok(Type::Class(Ident(s.clone()))),
            _ => Err(self.unexpected_token(tok.clone())),
        })?;
        tokens.next();

        let name = self.ident(tokens)?;
        // (
        let _ = self.skip_symbol(tokens, Symbol::LParen)?;
        let params = self.parameter_list(tokens)?;
        // )
        let _ = self.skip_symbol(tokens, Symbol::RParen)?;
        let body = self.subroutine_body(tokens)?;

        let res = SubRoutineDec::new(modifier, ty, name, params, body);
        Ok(res)
    }

    fn param<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Param, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let ty = self.ty(tokens)?;
        let var_name = self.ident(tokens)?;
        Ok(Param(ty, var_name))
    }

    fn parameter_list<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<ParamList, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let head = self.param(tokens).ok();
        match head {
            None => return Ok(ParamList(None)),
            Some(_) => (),
        };

        // (, type varname)*
        let mut tail = vec![];
        loop {
            match self.skip_symbol(tokens, Symbol::Comma) {
                Ok(_) => match self.param(tokens) {
                    Ok(param) => {
                        tail.push(param);
                    }
                    Err(e) => return Err(e),
                },
                Err(_) => break,
            }
        }

        Ok(ParamList(Some((head.unwrap(), tail))))
    }

    fn subroutine_body<Tokens>(
        &self,
        tokens: &mut Peekable<Tokens>,
    ) -> Result<SubRoutineBody, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        // {
        let _ = self.skip_symbol(tokens, Symbol::LCurlyParen)?;
        // vardec*
        let mut var_decs = vec![];
        loop {
            match self.var_dec(tokens) {
                Ok(var_dec) => var_decs.push(var_dec),
                Err(_) => break,
            }
        }
        let stmts = self.stmts(tokens)?;
        // }
        let _ = self.skip_symbol(tokens, Symbol::RCurlyParen)?;
        let res = SubRoutineBody::new(var_decs, stmts);
        Ok(res)
    }

    fn var_dec<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<VarDec, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        // var
        let _ = tokens.peek().ok_or(self.eof()).and_then(|tok| match tok {
            Token {
                value: TokenKind::Keyword(k),
                ..
            } => match k {
                Keyword::Var => Ok(()),
                _ => Err(self.unexpected_token(tok.clone())),
            },
            _ => Err(self.unexpected_token(tok.clone())),
        })?;
        tokens.next();

        let ty = self.ty(tokens)?;
        let name = self.ident(tokens)?;
        let mut names = vec![];
        // (, varname)*
        loop {
            match self.skip_symbol(tokens, Symbol::Comma) {
                Ok(()) => match self.ident(tokens) {
                    Ok(ident) => {
                        names.push(ident);
                    }
                    // , の次は varnameであるべき
                    Err(e) => return Err(e),
                },
                Err(_) => break,
            }
        }
        // ;
        let _ = self.skip_symbol(tokens, Symbol::SemiColon)?;

        let res = VarDec::new(ty, name, names);
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::*;

    fn parse(input: &str) -> Ast {
        let path = PathBuf::from("hoge");
        let tokens = Lexer::new(&path).run(input).unwrap();
        Parser::new(&path).run(tokens).unwrap()
    }

    #[test]
    fn test_parse_class() {
        let actual = parse("\nclass akashikeyanage{ } \n");
        let expect = Class::new(Ident("akashikeyanage".to_owned()), vec![], vec![]);
        assert_eq!(actual, Ast { class: expect });
    }
}
