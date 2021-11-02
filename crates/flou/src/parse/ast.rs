#![allow(dead_code)]

use nom::{
    branch::{alt, permutation},
    bytes::complete::take_while,
    character::{
        complete::{anychar, char},
        is_alphabetic, is_alphanumeric,
    },
    combinator::{map, opt, recognize, value, verify},
    multi::many1,
    sequence::{pair, preceded, separated_pair, terminated, tuple},
    Parser,
};
use nom_supreme::{final_parser::final_parser, tag::complete::tag, ParserExt};

use crate::{
    parse::combinators::enclosed_list0,
    pos::{pos, IndexPos},
};

use super::{
    combinators::{attribute, block, list1, space, ws},
    constants::*,
    parts::quoted_string,
    types::{Input, Result},
    Error,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Identifier<'i>(pub(crate) &'i str);

impl<'i> Identifier<'i> {
    pub(crate) fn parse(i: Input<'i>) -> Result<Self> {
        let wchar = take_while(|x: char| x == '_' || is_alphanumeric(x as u8));
        map(
            recognize(pair(
                verify(anychar, |&c| c == '_' || is_alphabetic(c as u8)),
                wchar,
            )),
            Self,
        )(i)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeShape {
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
pub enum Direction {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Destination<'i> {
    Itself,
    Relative(Direction),
    Label(Identifier<'i>),
}

impl<'i> Destination<'i> {
    pub(crate) fn parse(i: Input<'i>) -> Result<Self> {
        alt((
            preceded(
                char(RELATIVE_SIGIL),
                map(opt(Direction::parse), |dir| match dir {
                    Some(dir) => Self::Relative(dir),
                    None => Self::Itself,
                }),
            ),
            map(preceded(char(LABEL_SIGIL), Identifier::parse), Self::Label),
        ))(i)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum NodeAttribute<'i> {
    Text(String),
    Class(String),
    Shape(NodeShape),
    Connect(Vec<ConnectionDescriptor<'i>>),
}

impl<'i> NodeAttribute<'i> {
    fn parse(i: Input<'i>) -> Result<Self> {
        let connection_descriptors = alt((
            map(ConnectionDescriptor::parse, |x| vec![x]),
            enclosed_list0(BLOCK_DELIMITERS, ConnectionDescriptor::parse, TERMINATOR),
        ));

        alt((
            map(attribute("text", quoted_string), Self::Text),
            map(attribute("class", quoted_string), Self::Class),
            map(attribute("shape", NodeShape::parse), Self::Shape),
            map(attribute("connect", connection_descriptors), Self::Connect),
        ))(i)
    }

    fn parse_vec(i: Input<'i>) -> Result<Vec<Self>> {
        alt((
            map(quoted_string.terminated(opt(char(LIST_SEPARATOR))), |x| {
                vec![Self::Text(x)]
            })
            .terminated(char(LIST_DELIMITERS.1)),
            map(
                pair(
                    quoted_string.terminated(ws(char(LIST_SEPARATOR))),
                    list1(Self::parse, LIST_SEPARATOR, LIST_DELIMITERS.1),
                ),
                |(text_shorthand, mut tail)| {
                    tail.insert(0, Self::Text(text_shorthand));
                    tail
                },
            ),
            list1(Self::parse, LIST_SEPARATOR, LIST_DELIMITERS.1),
        ))
        .preceded_by(char(LIST_DELIMITERS.0))
        .parse(i)
    }

    pub(crate) fn as_key(&self) -> &'static str {
        match self {
            NodeAttribute::Text(_) => "text",
            NodeAttribute::Class(_) => "class",
            NodeAttribute::Shape(_) => "shape",
            NodeAttribute::Connect(_) => "connect",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct ConnectionDescriptor<'i> {
    pub(crate) to: Destination<'i>,
    pub(crate) sides: (Direction, Direction),
    pub(crate) attrs: Vec<ConnectionAttribute>,
}

impl<'i> ConnectionDescriptor<'i> {
    pub(crate) fn parse(i: Input<'i>) -> Result<Self> {
        let sides = separated_pair(Direction::parse, char(SIDES_SIGIL), Direction::parse);

        map(
            tuple((
                sides,
                Destination::parse,
                opt(ConnectionAttribute::parse_vec),
            )),
            |(sides, to, attrs)| Self {
                to,
                sides,
                attrs: attrs.unwrap_or_default(),
            },
        )(i)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum ArrowheadType {
    None,
    Start,
    End,
    Both,
}

impl Default for ArrowheadType {
    fn default() -> Self {
        Self::End
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum ConnectionAttribute {
    Text(String),
    Class(String),
    Arrowheads(ArrowheadType),
}

impl ConnectionAttribute {
    pub(crate) fn parse(i: Input) -> Result<Self> {
        let arrowheads = alt((
            value(ArrowheadType::None, tag("none")),
            value(ArrowheadType::Start, tag("start")),
            value(ArrowheadType::End, tag("end")),
            value(ArrowheadType::Both, tag("both")),
        ));

        alt((
            map(attribute("text", quoted_string), Self::Text),
            map(attribute("class", quoted_string), Self::Class),
            map(attribute("arrowheads", arrowheads), Self::Arrowheads),
        ))(i)
    }

    pub(crate) fn parse_vec(i: Input) -> Result<Vec<Self>> {
        alt((
            map(quoted_string, |x| vec![Self::Text(x)]).terminated(char(LIST_DELIMITERS.1)),
            map(
                pair(
                    quoted_string.terminated(ws(char(LIST_SEPARATOR))),
                    list1(Self::parse, LIST_SEPARATOR, LIST_DELIMITERS.1),
                ),
                |(text_shorthand, mut tail)| {
                    tail.insert(0, Self::Text(text_shorthand));
                    tail
                },
            ),
            list1(Self::parse, LIST_SEPARATOR, LIST_DELIMITERS.1),
        ))
        .preceded_by(char(LIST_DELIMITERS.0))
        .parse(i)
    }

    pub(crate) fn as_key(&self) -> &'static str {
        match self {
            Self::Text(_) => "text",
            Self::Class(_) => "class",
            Self::Arrowheads(_) => "arrowheads",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Node<'i> {
    pub(crate) id: Identifier<'i>,
    pub(crate) label: Option<Identifier<'i>>,
    pub(crate) attrs: Vec<NodeAttribute<'i>>,
}

impl<'i> Node<'i> {
    pub(crate) fn parse(i: Input<'i>) -> Result<Self> {
        map(
            tuple((
                Identifier::parse,
                opt(preceded(char(LABEL_SIGIL), Identifier::parse)),
                opt(NodeAttribute::parse_vec),
            )),
            |(id, label, attrs)| Self {
                id,
                label,
                attrs: attrs.unwrap_or_default(),
            },
        )(i)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Grid<'i>(Vec<Vec<Option<Node<'i>>>>);

impl<'i> Grid<'i> {
    pub(crate) fn parse(i: Input<'i>) -> Result<Self> {
        let empty = tag(EMPTY);
        let opt_node = alt((map(empty, |_| None), map(Node::parse, Some)));
        let row = list1(opt_node, LIST_SEPARATOR, TERMINATOR);
        let grid = map(many1(ws(row)), Self);

        preceded(terminated(tag("grid"), space), block(grid))(i)
    }

    pub(crate) fn nodes(&self) -> impl Iterator<Item = (IndexPos, &Node<'i>)> {
        self.0.iter().enumerate().flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                .filter_map(move |(x, node)| node.as_ref().map(|node| (pos(x, y).into(), node)))
        })
    }

    pub(crate) fn size(&self) -> IndexPos {
        let height = self.0.len();
        let width = self.0.iter().map(|v| v.len()).max().unwrap_or_default();

        pos(width, height).into()
    }
}

pub(crate) type Definitions<'i> = Vec<(Identifier<'i>, Vec<NodeAttribute<'i>>)>;

pub(crate) fn parse_definitions(i: Input) -> Result<Definitions> {
    let definition = pair(Identifier::parse, NodeAttribute::parse_vec).terminated(char(TERMINATOR));
    let definitions = many1(ws(definition));

    preceded(terminated(tag("define"), space), block(definitions))(i)
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Document<'i> {
    pub(crate) grid: Grid<'i>,
    pub(crate) definitions: Definitions<'i>,
}

impl<'i> Document<'i> {
    pub(crate) fn parse(i: Input<'i>) -> std::result::Result<Self, Error<'i>> {
        let document = map(
            permutation((ws(Grid::parse), opt(ws(parse_definitions)))),
            |(grid, definitions)| Self {
                grid,
                definitions: definitions.unwrap_or_default(),
            },
        );

        final_parser(document)(i)
    }
}

#[cfg(test)]
mod tests {
    use nom::combinator::all_consuming;

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
        );

        assert_parsed_eq(Destination::parse, "@", Destination::Itself);
    }

    #[test]
    fn valid_node_attribute() {
        assert_parsed_eq(
            NodeAttribute::parse,
            r#"text: "foo""#,
            NodeAttribute::Text(String::from("foo")),
        );

        assert_parsed_eq(
            NodeAttribute::parse,
            r#"class: "class name here""#,
            NodeAttribute::Class(String::from("class name here")),
        );

        assert_parsed_eq(
            NodeAttribute::parse,
            r#"shape: diamond"#,
            NodeAttribute::Shape(NodeShape::Diamond),
        );
    }

    #[test]
    fn valid_connection_attribute() {
        assert_parsed_eq(
            ConnectionAttribute::parse,
            r#"text: "foo""#,
            ConnectionAttribute::Text(String::from("foo")),
        );

        assert_parsed_eq(
            ConnectionAttribute::parse,
            r#"class: "class name here""#,
            ConnectionAttribute::Class(String::from("class name here")),
        );

        assert_parsed_eq(
            ConnectionAttribute::parse,
            "arrowheads: none",
            ConnectionAttribute::Arrowheads(ArrowheadType::None),
        );
    }

    #[test]
    fn valid_connection_descriptor() {
        assert_parsed_eq(
            ConnectionDescriptor::parse,
            r#"n:s@s("foo", class: "bar")"#,
            ConnectionDescriptor {
                to: Destination::Relative(Direction::South),
                sides: (Direction::North, Direction::South),
                attrs: vec![
                    ConnectionAttribute::Text(String::from("foo")),
                    ConnectionAttribute::Class(String::from("bar")),
                ],
            },
        );

        assert_parsed_eq(
            ConnectionDescriptor::parse,
            "w:e@s",
            ConnectionDescriptor {
                to: Destination::Relative(Direction::South),
                sides: (Direction::West, Direction::East),
                attrs: vec![],
            },
        );

        assert_parsed_eq(
            ConnectionDescriptor::parse,
            "n:e@s",
            ConnectionDescriptor {
                to: Destination::Relative(Direction::South),
                sides: (Direction::North, Direction::East),
                attrs: vec![],
            },
        );
    }

    #[test]
    fn valid_node_connect_attribute() {
        assert_parsed_eq(
            NodeAttribute::parse,
            "connect: n:e@n",
            NodeAttribute::Connect(vec![ConnectionDescriptor {
                to: Destination::Relative(Direction::North),
                sides: (Direction::North, Direction::East),
                attrs: vec![],
            }]),
        );

        assert_parsed_eq(
            NodeAttribute::parse,
            "connect: {n:n@e; n:n#foo}",
            NodeAttribute::Connect(vec![
                ConnectionDescriptor {
                    to: Destination::Relative(Direction::East),
                    sides: (Direction::North, Direction::North),
                    attrs: vec![],
                },
                ConnectionDescriptor {
                    to: Destination::Label(Identifier("foo")),
                    sides: (Direction::North, Direction::North),
                    attrs: vec![],
                },
            ]),
        )
    }

    #[test]
    fn valid_node() {
        assert_parsed_eq(
            Node::parse,
            "foo",
            Node {
                id: Identifier("foo"),
                label: None,
                attrs: vec![],
            },
        );

        assert_parsed_eq(
            Node::parse,
            "foo#bar(shape: rect)",
            Node {
                id: Identifier("foo"),
                label: Some(Identifier("bar")),
                attrs: vec![NodeAttribute::Shape(NodeShape::Rectangle)],
            },
        );

        assert_parsed_eq(
            Node::parse,
            r#"foo("hello",)"#,
            Node {
                id: Identifier("foo"),
                label: None,
                attrs: vec![NodeAttribute::Text(String::from("hello"))],
            },
        );

        assert_parsed_eq(
            Node::parse,
            r#"foo("hey", shape: diamond,)"#,
            Node {
                id: Identifier("foo"),
                label: None,
                attrs: vec![
                    NodeAttribute::Text(String::from("hey")),
                    NodeAttribute::Shape(NodeShape::Diamond),
                ],
            },
        );

        assert_parsed_eq(
            Node::parse,
            "foo#bar(shape: rect)",
            Node {
                id: Identifier("foo"),
                label: Some(Identifier("bar")),
                attrs: vec![NodeAttribute::Shape(NodeShape::Rectangle)],
            },
        );
    }

    #[test]
    fn invalid_node() {
        assert_not_parsed(Node::parse, "");
        assert_not_parsed(Node::parse, "(shape: rect)");
        assert_not_parsed(Node::parse, "#bar");
        assert_not_parsed(Node::parse, "#bar(shape: rect)");
        // Without all_consuming the parser just stops once it reaches "()".
        assert_not_parsed(all_consuming(Node::parse), "foo()");
    }

    #[test]
    fn valid_grid() {
        let input = r#"
            grid {
                foo#main, bar;
                baz, _;
                _;
            }
        "#
        .trim();

        let foo_node = Node {
            id: Identifier("foo"),
            label: Some(Identifier("main")),
            attrs: vec![],
        };
        let bar_node = Node {
            id: Identifier("bar"),
            label: None,
            attrs: vec![],
        };
        let baz_node = Node {
            id: Identifier("baz"),
            label: None,
            attrs: vec![],
        };

        assert_parsed_eq(
            Grid::parse,
            input,
            Grid(vec![
                vec![Some(foo_node), Some(bar_node)],
                vec![Some(baz_node), None],
                vec![None],
            ]),
        );
    }

    #[test]
    fn invalid_grid() {
        assert_not_parsed(Grid::parse, "grid {}");
        assert_not_parsed(Grid::parse, "grid { missing_terminator }");
        assert_not_parsed(Grid::parse, "grid { missing separator; }");
        assert_not_parsed(Grid::parse, "grid { foo; ; }");
    }

    #[test]
    fn valid_definitions() {
        let input = r#"
            define {
                foo(shape: rect);
                bar(text: "hello");
            }
        "#
        .trim();

        assert_parsed_eq(
            parse_definitions,
            input,
            vec![
                (
                    Identifier("foo"),
                    vec![NodeAttribute::Shape(NodeShape::Rectangle)],
                ),
                (
                    Identifier("bar"),
                    vec![NodeAttribute::Text(String::from("hello"))],
                ),
            ],
        )
    }

    #[test]
    fn invalid_definitions() {
        assert_not_parsed(parse_definitions, "define {}");
        assert_not_parsed(parse_definitions, "define { no_attrs; }");
        assert_not_parsed(parse_definitions, "define { no_terminator(shape: rect) }");
        assert_not_parsed(parse_definitions, "define { ; }");
    }
}
