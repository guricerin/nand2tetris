use super::*;

impl Parser {
    pub fn stmts<Tokens>(&self, tokens: &mut Peekable<Tokens>) -> Result<Stmts, ParseError>
    where
        Tokens: Iterator<Item = Token>,
    {
        todo!();
    }
}
