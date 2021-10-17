#![allow(dead_code)]

use nom::{
    branch::alt,
    bytes::complete::take_while,
    character::{
        complete::{anychar, char},
        is_alphabetic, is_alphanumeric,
    },
    combinator::{map, recognize, value, verify},
    sequence::{pair, preceded},
};
use nom_supreme::tag::complete::tag;

use super::{
    combinators::attribute,
    parts::quoted_string,
    types::{Input, Result},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Identifier<'i>(pub(crate) &'i str);

impl<'i> Identifier<'i> {
    pub(crate) fn parse(i: Input<'i>) -> Result<Self> {
        let wchar = take_while(|x: char| x == '_' || is_alphanumeric(x as u8));
        map(
            recognize(pair(
                verify(anychar, |&c| c == '_' || is_alphabetic(c as u8)),
                wchar,
            )),
            Identifier,
        )(i)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeShape {
    Rectangle,
    Square,
    Ellipse,
    Circle,
    Diamond,
    AngledSquare,
}

impl NodeShape {
    pub(crate) fn parse(i: Input) -> Result<Self> {
        alt((
            value(Self::Rectangle, tag("rect")),
            value(Self::Square, tag("square")),
            value(Self::Ellipse, tag("ellipse")),
            value(Self::Circle, tag("circle")),
            value(Self::Diamond, tag("diamond")),
            value(Self::AngledSquare, tag("angled_square")),
        ))(i)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Direction {
    North,
    South,
    West,
    East,
}

impl Direction {
    pub(crate) fn parse(i: Input) -> Result<Self> {
        alt((
            value(Self::North, tag("n")),
            value(Self::South, tag("s")),
            value(Self::West, tag("w")),
            value(Self::East, tag("e")),
        ))(i)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Destination<'i> {
    Relative(Direction),
    Label(Identifier<'i>),
}

impl<'i> Destination<'i> {
    pub(crate) fn parse(i: Input<'i>) -> Result<Self> {
        alt((
            map(preceded(char('@'), Direction::parse), Self::Relative),
            map(preceded(char('#'), Identifier::parse), Self::Label),
        ))(i)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum NodeAttribute {
    Text(String),
    Class(String),
    Shape(NodeShape),
}

impl NodeAttribute {
    pub(crate) fn parse(i: Input) -> Result<Self> {
        alt((
            map(attribute("text", quoted_string), Self::Text),
            map(attribute("class", quoted_string), Self::Class),
            map(attribute("shape", NodeShape::parse), Self::Shape),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{assert_not_parsed, assert_parsed_eq};

    #[test]
    fn valid_identifier() {
        assert_parsed_eq(Identifier::parse, "foo", Identifier("foo"));
        assert_parsed_eq(Identifier::parse, "bar21foo", Identifier("bar21foo"));
        assert_parsed_eq(Identifier::parse, "_example", Identifier("_example"));
        assert_parsed_eq(Identifier::parse, "text_14", Identifier("text_14"));
    }

    #[test]
    fn invalid_identifier() {
        assert_not_parsed(Identifier::parse, "");
        assert_not_parsed(Identifier::parse, "12number_first");
    }

    #[test]
    fn valid_node_shape() {
        assert_parsed_eq(NodeShape::parse, "rect", NodeShape::Rectangle);
        assert_parsed_eq(NodeShape::parse, "square", NodeShape::Square);
        assert_parsed_eq(NodeShape::parse, "ellipse", NodeShape::Ellipse);
        assert_parsed_eq(NodeShape::parse, "circle", NodeShape::Circle);
        assert_parsed_eq(NodeShape::parse, "diamond", NodeShape::Diamond);
        assert_parsed_eq(NodeShape::parse, "angled_square", NodeShape::AngledSquare);
    }

    #[test]
    fn valid_direction() {
        assert_parsed_eq(Direction::parse, "n", Direction::North);
        assert_parsed_eq(Direction::parse, "s", Direction::South);
        assert_parsed_eq(Direction::parse, "w", Direction::West);
        assert_parsed_eq(Direction::parse, "e", Direction::East);
    }

    #[test]
    fn valid_destination() {
        const NORTH: Destination = Destination::Relative(Direction::North);
        const SOUTH: Destination = Destination::Relative(Direction::South);
        const WEST: Destination = Destination::Relative(Direction::West);
        const EAST: Destination = Destination::Relative(Direction::East);

        assert_parsed_eq(Destination::parse, "@n", NORTH);
        assert_parsed_eq(Destination::parse, "@s", SOUTH);
        assert_parsed_eq(Destination::parse, "@w", WEST);
        assert_parsed_eq(Destination::parse, "@e", EAST);

        assert_parsed_eq(
            Destination::parse,
            "#foo",
            Destination::Label(Identifier("foo")),
        )
    }

    #[test]
    fn valid_node_attribute() {
        assert_parsed_eq(
            NodeAttribute::parse,
            r#"text: "foo""#,
            NodeAttribute::Text("foo".into()),
        );

        assert_parsed_eq(
            NodeAttribute::parse,
            r#"class: "class name here""#,
            NodeAttribute::Class("class name here".into()),
        );

        assert_parsed_eq(
            NodeAttribute::parse,
            r#"shape: diamond"#,
            NodeAttribute::Shape(NodeShape::Diamond),
        );
    }
}
