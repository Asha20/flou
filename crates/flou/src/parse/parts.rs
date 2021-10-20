use nom::{
    branch::alt,
    bytes::complete::escaped_transform,
    character::complete::{char, none_of},
    combinator::{cut, map, opt, value},
    sequence::delimited,
};
use nom_supreme::tag::complete::tag;

use super::{Input, Result};

pub(super) fn quoted_string(i: Input) -> Result<String> {
    let esc = escaped_transform(
        none_of("\\\""),
        '\\',
        alt((
            value("\\", tag("\\")),
            value("\"", tag("\"")),
            value("\n", tag("n")),
        )),
    );

    map(
        delimited(char('"'), cut(opt(esc)), cut(char('"'))),
        Option::unwrap_or_default,
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{assert_eq, assert_not_parsed, assert_parsed_eq};

    #[test]
    fn valid_quoted_string() {
        assert_parsed_eq(quoted_string, r#""example""#, "example".into());
    }

    #[test]
    fn quoted_string_escapes_characters() {
        assert_parsed_eq(
            quoted_string,
            r#""I said \"hello\".\n""#,
            "I said \"hello\".\n".into(),
        );
    }

    #[test]
    fn invalid_quoted_string() {
        assert_not_parsed(quoted_string, r#""missing end quote"#);
        assert_not_parsed(quoted_string, r#"missing start quote""#);
        assert_not_parsed(quoted_string, r#"missing both quotes"#);
    }
}
