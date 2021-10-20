use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
};

use crate::parse::{ast, Input, Parser};

pub(crate) use pretty_assertions::assert_eq;

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

pub(crate) fn map<K: Eq + std::hash::Hash, V, I: IntoIterator<Item = (K, V)>>(
    xs: I,
) -> HashMap<K, V> {
    HashMap::from_iter(xs)
}

pub(crate) fn set<T: Eq + std::hash::Hash, I: IntoIterator<Item = T>>(xs: I) -> HashSet<T> {
    HashSet::from_iter(xs)
}
