use crate::parse::{Input, Parser};

pub(crate) fn assert_parsed_eq<'i, P: Parser<'i, O>, O: std::fmt::Debug + PartialEq>(
    mut parser: P,
    input: Input<'i>,
    expected: O,
) {
    let actual = parser.parse(input);
    assert!(actual.is_ok(), "Parser should not fail");

    let actual = actual.unwrap();
    assert_eq!(
        actual.1, expected,
        "Parsed value does not match expected value"
    );
    assert_eq!(actual.0, "", "There should be no more input");
}

pub(crate) fn assert_not_parsed<'i, P: Parser<'i, O>, O: std::fmt::Debug + PartialEq>(
    mut parser: P,
    input: Input<'i>,
) {
    let actual = parser.parse(input);
    assert!(actual.is_err(), "Parser should fail");
}
