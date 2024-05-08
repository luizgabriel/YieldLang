use std::fmt::Debug;
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::{is_not, take_while_m_n};
use nom::character::complete::{char, line_ending, multispace1, space0, space1};
use nom::combinator::{map, map_opt, map_res, value, verify};
use nom::error::{context, VerboseError};
use nom::multi::{fold_many0, many1, separated_list1};
use nom::sequence::{delimited, preceded, separated_pair};
use nom::IResult;
use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, digit1},
    combinator::{eof, not, recognize},
    sequence::{pair, terminated},
};

use crate::ast::{Expr, LiteralValue};

type ParseResult<'a, R> = IResult<&'a str, R, VerboseError<&'a str>>;

fn boolean<'a>(input: &'a str) -> ParseResult<'a, bool> {
    alt((value(true, tag("true")), value(false, tag("false"))))(input)
}

fn integer<'a, O>(input: &'a str) -> ParseResult<'a, O>
where
    O: FromStr,
    <O as FromStr>::Err: Debug,
{
    map(recognize(digit1), |i: &'a str| i.parse().unwrap())(input)
}

fn identifier<'a>(input: &'a str) -> ParseResult<'a, &'a str> {
    recognize(pair(not(digit1), many1(alt((alphanumeric1, tag("_"))))))(input)
}

fn string_unicode<'a>(input: &'a str) -> ParseResult<'a, char> {
    let parse_hex = take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit());
    let parse_delimited_hex = preceded(char('u'), delimited(char('{'), parse_hex, char('}')));
    let parse_u32 = map_res(parse_delimited_hex, move |hex| u32::from_str_radix(hex, 16));
    map_opt(parse_u32, std::char::from_u32)(input)
}

fn string_escaped_char<'a>(input: &'a str) -> ParseResult<'a, char> {
    preceded(
        char('\\'),
        alt((
            string_unicode,
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\u{08}', char('b')),
            value('\u{0C}', char('f')),
            value('\\', char('\\')),
            value('/', char('/')),
            value('"', char('"')),
        )),
    )(input)
}

fn string_escaped_whitespace<'a>(input: &'a str) -> ParseResult<'a, &'a str> {
    preceded(char('\\'), multispace1)(input)
}

fn string_literal<'a>(input: &'a str) -> ParseResult<'a, &'a str> {
    let not_quote_slash = is_not("\"\\");
    verify(not_quote_slash, |s: &str| !s.is_empty())(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWS,
}

fn string_fragment<'a>(input: &'a str) -> ParseResult<'a, StringFragment> {
    alt((
        map(string_literal, StringFragment::Literal),
        map(string_escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, string_escaped_whitespace),
    ))(input)
}

fn string<'a>(input: &'a str) -> ParseResult<'a, String> {
    let build_string = fold_many0(string_fragment, String::new, |mut string, fragment| {
        match fragment {
            StringFragment::Literal(s) => string.push_str(s),
            StringFragment::EscapedChar(c) => string.push(c),
            StringFragment::EscapedWS => {}
        }
        string
    });

    delimited(char('"'), build_string, char('"'))(input)
}

fn literals<'a>(input: &'a str) -> ParseResult<'a, LiteralValue> {
    alt((
        map(integer, LiteralValue::Integer),
        map(string, LiteralValue::String),
        map(boolean, LiteralValue::Boolean),
    ))(input)
}

fn expr_p0<'a>(input: &'a str) -> ParseResult<'a, Expr> {
    context(
        "terminals",
        alt((
            map(literals, Expr::Literal),
            map(identifier, |i: &'a str| Expr::Identifier(i.to_owned())),
        )),
    )(input)
}

fn expr_p1<'a>(input: &'a str) -> ParseResult<'a, Expr> {
    let paren_expr = delimited(char('('), expr, char(')'));
    context("parenthesized expr", alt((paren_expr, expr_p0)))(input)
}

fn expr_p2<'a>(input: &'a str) -> ParseResult<'a, Expr> {
    let fn_app = context(
        "function application",
        map(
            separated_pair(expr_p1, space1, separated_list1(space1, expr_p1)),
            |(head, tail)| {
                tail.into_iter().fold(head, |lhs, rhs| {
                    Expr::FunctionApplication(Box::new(lhs), Box::new(rhs))
                })
            },
        ),
    );

    alt((fn_app, expr_p1))(input)
}

fn expr_p3<'a>(input: &'a str) -> ParseResult<'a, Expr> {
    let assign_exp = context(
        "assignment expr",
        map(
            separated_pair(identifier, delimited(space1, char('='), space1), expr_p2),
            |(id, body): (&'a str, Expr)| Expr::AssignmentExpr(id.to_owned(), Box::new(body)),
        ),
    );

    alt((assign_exp, expr_p2))(input)
}

fn expr<'a>(input: &'a str) -> ParseResult<'a, Expr> {
    context("expr", delimited(space0, expr_p3, space0))(input)
}

fn end_of_statement<'a>(input: &'a str) -> ParseResult<'a, ()> {
    context("end of statement", value((), many1(line_ending)))(input)
}

pub fn program<'a>(input: &'a str) -> ParseResult<'a, Expr> {
    context(
        "program",
        map(
            terminated(separated_list1(end_of_statement, expr), eof),
            Expr::Block,
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Finish;

    macro_rules! assert_parse {
        ($input:expr, $expected:pat) => {
            assert!(matches!($input.finish(), Ok((_, $expected))));
        };
    }

    macro_rules! assert_parse_err {
        ($input:expr) => {
            assert!($input.finish().is_err());
        };
    }

    #[test]
    fn test_integer() {
        assert_parse!(integer::<u32>("123"), 123);
        assert_parse_err!(integer::<u32>("abc"));
    }

    #[test]
    fn test_boolean() {
        assert_parse!(boolean("true"), true);
        assert_parse!(boolean("false"), false);
        assert_parse_err!(boolean("abc"));
    }

    #[test]
    fn test_identifier() {
        assert_parse!(identifier("abc"), "abc");
        assert_parse!(identifier("abc123"), "abc123");
        assert_parse!(identifier("abc_123"), "abc_123");
        assert_parse!(identifier("_abc_123"), "_abc_123");
        assert_parse_err!(identifier(""));
        assert_parse_err!(identifier("123abc"));
    }
}
