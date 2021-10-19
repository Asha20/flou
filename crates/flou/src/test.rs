use crate::parse::{ast, Input, Parser};

pub(crate) fn assert_parsed_eq<'i, P: Parser<'i, O>, O: std::fmt::Debug + PartialEq>(
    mut parser: P,
    input: Input<'i>,
    expected: O,
) {
    let actual = parser.parse(input);
    assert!(actual.is_ok(), "Unexpected error: {}", actual.unwrap_err());

    let actual = actual.unwrap();
    assert_eq!(
        actual.1, expected,
        "Parsed value does not match expected value"
    );
    assert_eq!(actual.0, "", "Unexpected input: {:?}", actual.0);
}

pub(crate) fn assert_not_parsed<'i, P: Parser<'i, O>, O: std::fmt::Debug + PartialEq>(
    mut parser: P,
    input: Input<'i>,
) {
    let actual = parser.parse(input);
    assert!(actual.is_err(), "Unexpected success: {:?}", actual.unwrap());
}

pub(crate) fn id(s: &str) -> ast::Identifier {
    ast::Identifier(s)
}

macro_rules! map {
    ($($k:expr => $v:expr),* $(,)?) => {{
        use std::iter::{FromIterator, IntoIterator};
        use std::collections::hash_map::HashMap;
        HashMap::from_iter(IntoIterator::into_iter([$(($k, $v),)*]))
    }};
}

macro_rules! set {
    ($($v:expr),* $(,)?) => {{
        use std::iter::{FromIterator, IntoIterator};
        use std::collections::hash_set::HashSet;
        HashSet::from_iter(IntoIterator::into_iter([$($v,)*]))
    }};
}

pub(crate) use map;
pub(crate) use set;
