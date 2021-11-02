use nom::{
    branch::alt,
    character::complete::{char, multispace0},
    combinator::{cut, map},
    sequence::{delimited, pair, preceded},
};
use nom_supreme::{multi::collect_separated_terminated, tag::complete::tag, ParserExt};

use super::{constants::BLOCK_DELIMITERS, Parser};

/// Parses an item surrounded by `multispace0`.
pub(super) fn ws<'i, O, P: Parser<'i, O>>(item: P) -> impl Parser<'i, O> {
    delimited(multispace0, item, multispace0)
}

pub(super) fn attribute<'i, O, V: Parser<'i, O>>(
    key: &'static str,
    value: V,
) -> impl Parser<'i, O> {
    preceded(pair(tag(key), cut(ws(char(':')))), cut(value))
}

pub(super) fn enclosed_list0<'i, Item, Separator>(
    delimiters: (char, char),
    item: impl Parser<'i, Item>,
    separator: impl Parser<'i, Separator>,
) -> impl Parser<'i, Vec<Item>> {
    preceded(
        char(delimiters.0).terminated(multispace0),
        alt((
            map(char(delimiters.1).preceded_by(multispace0), |_| {
                Vec::default()
            }),
            collect_separated_terminated(
                item,
                ws(separator),
                char(delimiters.1).preceded_by(multispace0),
            ),
        )),
    )
}

pub(super) fn list1<'i, Item, Separator, Terminator>(
    item: impl Parser<'i, Item>,
    separator: impl Parser<'i, Separator>,
    terminator: impl Parser<'i, Terminator>,
) -> impl Parser<'i, Vec<Item>> {
    collect_separated_terminated(item, ws(separator), terminator.preceded_by(multispace0))
}

pub(super) fn block<'i, O, P: Parser<'i, O>>(item: P) -> impl Parser<'i, O> {
    delimited(char(BLOCK_DELIMITERS.0), ws(item), char(BLOCK_DELIMITERS.1))
}
