pub mod ast;
mod expression;
mod statement;

use crate::lex::token::*;
use ast::*;
use std::iter::Peekable;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("unexpected token\nFile: {0}\n{1}")]
    UnexpectedToken(PathBuf, Token),
    #[error("unexpected eof\nFile: {0}")]
    Eof(PathBuf),
    #[error("redundant token\nFile: {0}\n{1}")]
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
        self.past(&mut tokens)
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

    fn past<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Ast, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let class = self.pclass(tokens)?;
        match tokens.next() {
            None => Ok(Ast { class }),
            // 1ファイル1クラス想定
            Some(t) => Err(self.redundant_token(t)),
        }
    }

    fn pclass<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Class, ParseError>
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
                let name = self.pident(tokens)?;
                // {
                let _ = self.skip_symbol(tokens, Symbol::LCurlyParen)?;
                // vardec*
                let mut var_decs = vec![];
                loop {
                    match self.pclass_var_dec(tokens) {
                        Ok(decs) => {
                            var_decs.push(decs);
                        }
                        Err(_) => break,
                    }
                }
                // subroutinedec*
                let mut subroutine_decs = vec![];
                loop {
                    match self.psubroutine_dec(tokens) {
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

    fn pident<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Ident, ParseError>
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

    fn pclass_var_dec<Tokens>(
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

        let ty = self.pty(tokens)?;
        let name = self.pident(tokens)?;
        // (, ident)*
        let mut names = vec![];
        loop {
            match self.skip_symbol(tokens, Symbol::Comma) {
                Ok(()) => match self.pident(tokens) {
                    Ok(ident) => {
                        names.push(ident);
                    }
                    // , の次はvarnameであるべき
                    Err(e) => return Err(e),
                },
                Err(_) => break,
            }
        }
        let res = ClassVarDec::new(modifier, ty, name, names);
        tokens.next();
        Ok(res)
    }

    fn pty<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Type, ParseError>
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

    fn psubroutine_dec<Tokens>(
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

        let name = self.pident(tokens)?;
        // (
        let _ = self.skip_symbol(tokens, Symbol::LParen)?;
        let params = self.pparameter_list(tokens)?;
        // )
        let _ = self.skip_symbol(tokens, Symbol::RParen)?;
        let body = self.psubroutine_body(tokens)?;

        let res = SubRoutineDec::new(modifier, ty, name, params, body);
        Ok(res)
    }

    fn pparam<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Param, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let ty = self.pty(tokens)?;
        let var_name = self.pident(tokens)?;
        Ok(Param(ty, var_name))
    }

    fn pparameter_list<Tokens>(
        &self,
        tokens: &mut Peekable<Tokens>,
    ) -> Result<ParamList, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        let head = self.pparam(tokens).ok();
        match head {
            None => return Ok(ParamList(None)),
            Some(_) => (),
        };

        // (, type varname)*
        let mut tail = vec![];
        loop {
            match self.skip_symbol(tokens, Symbol::Comma) {
                Ok(_) => match self.pparam(tokens) {
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

    fn psubroutine_body<Tokens>(
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
            match self.pvar_dec(tokens) {
                Ok(var_dec) => var_decs.push(var_dec),
                Err(_) => break,
            }
        }
        let stmts = self.pstmts(tokens)?;
        // }
        let _ = self.skip_symbol(tokens, Symbol::RCurlyParen)?;
        let res = SubRoutineBody::new(var_decs, stmts);
        Ok(res)
    }

    fn pvar_dec<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<VarDec, ParseError>
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

        let ty = self.pty(tokens)?;
        let name = self.pident(tokens)?;
        let mut names = vec![];
        // (, varname)*
        loop {
            match self.skip_symbol(tokens, Symbol::Comma) {
                Ok(()) => match self.pident(tokens) {
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
    fn test_parse_empty_class() {
        let actual = parse("\nclass akashikeyanage{ } \n");
        let expect = Class::new(Ident("akashikeyanage".to_owned()), vec![], vec![]);
        assert_eq!(actual, Ast { class: expect });
    }

    #[test]
    #[ignore]
    fn test_parse_expressionless_class() {
        let input = r#"
// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/10/ExpressionLessSquare/Main.jack

/** Expressionless version of projects/10/Square/Main.jack. */

class Main {
    static boolean test;    // Added for testing -- there is no static keyword
                            // in the Square files.

    function void main() {
        var SquareGame game;
        let game = game;
        do game.run();
        do game.dispose();
        return;
    }

    function void test() {  // Added to test Jack syntax that is not use in
        var int i, j;       // the Square files.
        var String s;
        var Array a;
        if (i) {
            let s = i;
            let s = j;
            let a[i] = j;
        }
        else {              // There is no else keyword in the Square files.
            let i = i;
            let j = j;
            let i = i | j;
        }
        return;
    }
}
        "#;
        let actual = parse(input);
        let expect = Class::new(Ident("akashikeyanage".to_owned()), vec![], vec![]);
        assert_eq!(actual, Ast { class: expect });
    }
}
