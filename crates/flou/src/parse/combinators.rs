use nom::{
    character::complete::{char, multispace0},
    combinator::cut,
    sequence::{delimited, pair, preceded},
};
use nom_supreme::{multi::collect_separated_terminated, tag::complete::tag, ParserExt};

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

pub(crate) fn enclosed_list1<'i, Item, Separator>(
    delimiters: (char, char),
    item: impl Parser<'i, Item>,
    separator: impl Parser<'i, Separator>,
) -> impl Parser<'i, Vec<Item>> {
    preceded(
        char(delimiters.0).terminated(multispace0),
        collect_separated_terminated(
            item,
            ws(separator),
            char(delimiters.1).preceded_by(multispace0),
        ),
    )
}
