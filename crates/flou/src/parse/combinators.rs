use nom::{
    branch::alt,
    character::complete::{char, line_ending, multispace0, not_line_ending},
    combinator::{cut, map, opt, recognize, value},
    sequence::{delimited, pair, preceded},
};
use nom_supreme::{multi::collect_separated_terminated, tag::complete::tag, ParserExt};

use super::{constants::BLOCK_DELIMITERS, Input, Parser, Result};

fn comment(i: Input) -> Result<Input> {
    delimited(tag("//"), not_line_ending, line_ending)(i)
}

pub(super) fn space(i: Input) -> Result<()> {
    delimited(multispace0, value((), opt(comment)), multispace0)(i)
}

/// Parses an item surrounded by space and optional comments.
pub(super) fn ws<'i, O, P: Parser<'i, O>>(item: P) -> impl Parser<'i, O> {
    delimited(space, item, space)
}

pub(super) fn attribute<'i, O, V: Parser<'i, O>>(
    key: &'static str,
    value: V,
) -> impl Parser<'i, O> {
    preceded(pair(tag(key), cut(ws(char(':')))), cut(value))
}

pub(super) fn enclosed_list0<'i, Item>(
    delimiters: (char, char),
    item: impl Parser<'i, Item>,
    separator: char,
) -> impl Parser<'i, Vec<Item>> {
    preceded(
        char(delimiters.0).terminated(space),
        alt((
            map(char(delimiters.1).preceded_by(space), |_| Vec::default()),
            list1(item, separator, delimiters.1),
        )),
    )
}

pub(super) fn list1<'i, Item>(
    item: impl Parser<'i, Item>,
    separator: char,
    terminator: char,
) -> impl Parser<'i, Vec<Item>> {
    collect_separated_terminated(
        item,
        ws(char(separator)),
        alt((
            recognize(pair(char(separator), char(terminator).preceded_by(space))),
            recognize(char(terminator)),
        ))
        .preceded_by(space),
    )
}

pub(super) fn block<'i, O, P: Parser<'i, O>>(item: P) -> impl Parser<'i, O> {
    delimited(char(BLOCK_DELIMITERS.0), ws(item), char(BLOCK_DELIMITERS.1))
}
