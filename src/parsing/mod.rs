use self::{ast::Expr, grammar::*};
use nom::error::convert_error;
use nom::Finish;

pub mod ast;
mod grammar;

pub fn parse<'a>(input: &'a str) -> anyhow::Result<Expr> {
    program(input)
        .finish()
        .map_err(|e| anyhow::anyhow!(convert_error(input, e)))
        .map(|(_, r)| r)
}
