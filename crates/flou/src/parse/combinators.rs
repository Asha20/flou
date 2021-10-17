use nom::{
    character::complete::{char, multispace0},
    combinator::cut,
    sequence::{delimited, pair, preceded},
};
use nom_supreme::tag::complete::tag;

use super::Parser;

/// Parses an item surrounded by `multispace0`.
pub(crate) fn ws<'i, O, P: Parser<'i, O>>(item: P) -> impl Parser<'i, O> {
    delimited(multispace0, item, multispace0)
}

pub(crate) fn attribute<'i, O, V: Parser<'i, O>>(
    key: &'static str,
    value: V,
) -> impl Parser<'i, O> {
    preceded(pair(tag(key), cut(ws(char(':')))), cut(value))
}
