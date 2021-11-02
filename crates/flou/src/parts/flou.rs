use std::{
    collections::{hash_map, HashMap, HashSet},
    convert::TryFrom,
    fmt::Display,
};

use crate::{
    parse::ast::{
        ConnectionAttribute, ConnectionDescriptor, Destination, Direction, Document,
        Grid as ASTGrid, Identifier, NodeAttribute, NodeShape,
    },
    parse::Error as AstError,
    pos::IndexPos,
};

use super::{
    error::LogicError,
    grid::{Grid, ResolutionError},
};

type MapPos<T> = HashMap<IndexPos, T>;
type MapId<'i, T> = HashMap<Identifier<'i>, T>;
type TwoMapId<'i, T1, T2> = (MapId<'i, T1>, MapId<'i, T2>);
type TwoMapPos<T1, T2> = (MapPos<T1>, MapPos<T2>);

#[derive(Debug, Default, Clone)]
pub(crate) struct NodeAttributes {
    pub(crate) text: Option<String>,
    pub(crate) class: Option<String>,
    pub(crate) shape: Option<NodeShape>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ConnectionAttributes {
    pub(crate) text: Option<String>,
    pub(crate) class: Option<String>,
}

#[derive(Debug)]
pub(crate) struct Connection {
    pub(crate) from: (IndexPos, Direction),
    pub(crate) to: (IndexPos, Direction),
    pub(crate) attrs: ConnectionAttributes,
}

#[derive(Debug)]
pub struct Flou<'i> {
    pub(crate) grid: Grid<'i>,
    pub(crate) connections: Vec<Connection>,
    pub(crate) node_attributes: MapPos<NodeAttributes>,
}

#[derive(Debug)]
pub enum FlouError<'i> {
    Parse(AstError<'i>),
    Logic(LogicError<'i>),
}

impl<'i> TryFrom<&'i str> for Flou<'i> {
    type Error = FlouError<'i>;

    fn try_from(i: &'i str) -> Result<Self, Self::Error> {
        let document = Document::parse(i).map_err(FlouError::Parse)?;
        let flou = Flou::try_from(document).map_err(FlouError::Logic)?;
        Ok(flou)
    }
}

pub trait Renderer {
    fn render<'i>(&self, flou: &'i Flou<'i>) -> Box<dyn Display + 'i>;
}

impl<'i> TryFrom<Document<'i>> for Flou<'i> {
    type Error = LogicError<'i>;

    fn try_from(document: Document<'i>) -> Result<Self, Self::Error> {
        let grid = Grid::from(&document.grid);

        let definitions = ensure_definitions_are_unique(document.definitions)
            .map_err(LogicError::DuplicateDefinitions)?;

        // TODO: Warn if a definition doesn't map to any nodes in the grid.

        let (def_attrs, def_connections) = {
            let (def_attrs, def_connection_desc_map) = get_attributes_from_definitions(definitions)
                .map_err(LogicError::DuplicateNodeAttributesInDefinitions)?;

            let def_connections = parse_connection_desc_map(def_connection_desc_map)
                .map_err(LogicError::DuplicateConnectionAttributesInDefinitions)?;

            (
                resolve_id_map(&grid, def_attrs),
                resolve_id_map(&grid, def_connections),
            )
        };

        let (grid_attrs, grid_connections) = {
            let (grid_attrs, grid_conn_desc_map) = get_attributes_from_grid(&document.grid)
                .map_err(LogicError::DuplicateNodeAttributesInGrid)?;

            let grid_connections = parse_connection_desc_map(grid_conn_desc_map)
                .map_err(LogicError::DuplicateConnectionAttributesInGrid)?;

            (grid_attrs, grid_connections)
        };

        let node_attributes = Overwrite::overwrite(def_attrs, grid_attrs);
        let connections = Overwrite::overwrite(def_connections, grid_connections);

        let labels = try_into_label_map(&document.grid).map_err(LogicError::DuplicateLabels)?;

        let connections = resolve_connections_map(&grid, &labels, connections)
            .map_err(LogicError::InvalidDestination)?;

        Ok(Self {
            grid,
            connections,
            node_attributes,
        })
    }
}

/// Tries to assemble `NodeAttributes` from the vector of individual attributes.
/// The `connect` attribute is separated from the rest so that an independent
/// vector of connections can be created later down the line.
fn parse_node_attributes<'i>(
    attributes: Vec<NodeAttribute<'i>>,
) -> Result<(NodeAttributes, Option<Vec<ConnectionDescriptor<'i>>>), HashSet<&'static str>> {
    let mut res = NodeAttributes::default();
    let mut duplicates = HashSet::new();
    let mut conn_descriptors = None;

    for attribute in attributes {
        match attribute {
            NodeAttribute::Text(text) if res.text.is_none() => res.text = Some(text),
            NodeAttribute::Class(class) if res.class.is_none() => res.class = Some(class),
            NodeAttribute::Shape(shape) if res.shape.is_none() => res.shape = Some(shape),
            NodeAttribute::Connect(descriptors) if conn_descriptors.is_none() => {
                conn_descriptors = Some(descriptors)
            }
            _ => {
                duplicates.insert(attribute.as_key());
            }
        }
    }

    if duplicates.is_empty() {
        Ok((res, conn_descriptors))
    } else {
        Err(duplicates)
    }
}

impl TryFrom<Vec<ConnectionAttribute>> for ConnectionAttributes {
    type Error = HashSet<&'static str>;

    fn try_from(attributes: Vec<ConnectionAttribute>) -> Result<Self, Self::Error> {
        let mut res = Self::default();
        let mut duplicates = HashSet::new();

        for attribute in attributes {
            match attribute {
                ConnectionAttribute::Text(text) if res.text.is_none() => res.text = Some(text),
                ConnectionAttribute::Class(class) if res.class.is_none() => res.class = Some(class),
                _ => {
                    duplicates.insert(attribute.as_key());
                }
            }
        }

        if duplicates.is_empty() {
            Ok(res)
        } else {
            Err(duplicates)
        }
    }
}

/// Tries to map a label to the position of the node it's attached to.
/// Labels are supposed to be unique, so encountering duplicates is an error.
fn try_into_label_map<'i>(
    grid: &ASTGrid<'i>,
) -> Result<MapId<'i, IndexPos>, MapId<'i, HashSet<IndexPos>>> {
    let mut positions: HashMap<Identifier, HashSet<IndexPos>> = HashMap::new();

    for (pos, node) in grid.nodes() {
        if let Some(label) = node.label {
            positions.entry(label).or_default().insert(pos);
        }
    }

    let mut labels = HashMap::new();
    let mut unique_labels = HashSet::new();

    for (&label, positions) in &positions {
        if positions.len() == 1 {
            labels.insert(label, *positions.iter().next().unwrap());
            unique_labels.insert(label);
        }
    }

    if unique_labels.len() < positions.len() {
        for label in unique_labels {
            positions.remove(&label);
        }
        Err(positions)
    } else {
        Ok(labels)
    }
}

fn get_attributes_from_definitions<'i>(
    definitions: MapId<'i, Vec<NodeAttribute<'i>>>,
) -> Result<
    TwoMapId<'i, NodeAttributes, Vec<ConnectionDescriptor<'i>>>,
    MapId<'i, HashSet<&'static str>>,
> {
    let mut errors = HashMap::new();
    let mut map_node_attrs = HashMap::new();
    let mut map_connection_descriptors = HashMap::new();

    for (id, attrs) in definitions {
        match parse_node_attributes(attrs) {
            Ok((node_attrs, connection_descriptors)) => {
                map_node_attrs.insert(id, node_attrs);
                if let Some(descriptors) = connection_descriptors {
                    map_connection_descriptors.insert(id, descriptors);
                }
            }
            Err(duplicate_attrs) => {
                errors.insert(id, duplicate_attrs);
            }
        }
    }

    if errors.is_empty() {
        Ok((map_node_attrs, map_connection_descriptors))
    } else {
        Err(errors)
    }
}

fn get_attributes_from_grid<'i>(
    grid: &ASTGrid<'i>,
) -> Result<TwoMapPos<NodeAttributes, Vec<ConnectionDescriptor<'i>>>, MapPos<HashSet<&'static str>>>
{
    let mut errors = HashMap::new();
    let mut map_node_attrs = HashMap::new();
    let mut map_connection_descriptors = HashMap::new();

    for (pos, node) in grid.nodes() {
        match parse_node_attributes(node.attrs.clone()) {
            Ok((node_attrs, connection_descriptors)) => {
                map_node_attrs.insert(pos, node_attrs);
                if let Some(descriptors) = connection_descriptors {
                    map_connection_descriptors.insert(pos, descriptors);
                }
            }
            Err(duplicate_attrs) => {
                errors.insert(pos, duplicate_attrs);
            }
        }
    }

    if errors.is_empty() {
        Ok((map_node_attrs, map_connection_descriptors))
    } else {
        Err(errors)
    }
}

fn ensure_definitions_are_unique<'i>(
    definitions: Vec<(Identifier<'i>, Vec<NodeAttribute<'i>>)>,
) -> Result<MapId<Vec<NodeAttribute<'i>>>, HashSet<Identifier<'i>>> {
    let mut duplicates = HashSet::new();
    let mut res = HashMap::new();

    for (id, attrs) in definitions {
        if let hash_map::Entry::Vacant(e) = res.entry(id) {
            e.insert(attrs);
        } else {
            duplicates.insert(id);
        }
    }

    if duplicates.is_empty() {
        Ok(res)
    } else {
        Err(duplicates)
    }
}

#[derive(Debug, Clone)]
struct UnresolvedConnection<'i> {
    to: Destination<'i>,
    sides: (Direction, Direction),
    attrs: ConnectionAttributes,
}

type MapToUnresolvedConnection<'i, T> = HashMap<T, Vec<UnresolvedConnection<'i>>>;
type MapToDuplicateAttrs<'i, T> = HashMap<T, HashMap<usize, HashSet<&'static str>>>;

fn parse_connection_desc_map<T: Eq + std::hash::Hash + Copy>(
    def_connection_desc_map: HashMap<T, Vec<ConnectionDescriptor>>,
) -> Result<MapToUnresolvedConnection<T>, MapToDuplicateAttrs<T>> {
    let mut errors = HashMap::new();
    let mut res = HashMap::new();

    for (id, descriptors) in def_connection_desc_map {
        let mut value = Vec::new();
        for (i, descriptor) in descriptors.into_iter().enumerate() {
            match ConnectionAttributes::try_from(descriptor.attrs) {
                Ok(attrs) => {
                    value.push(UnresolvedConnection {
                        to: descriptor.to,
                        sides: descriptor.sides,
                        attrs,
                    });
                }
                Err(duplicate_attrs) => {
                    errors
                        .entry(id)
                        .or_insert_with(HashMap::new)
                        .insert(i, duplicate_attrs);
                }
            };
        }

        res.insert(id, value);
    }

    if errors.is_empty() {
        Ok(res)
    } else {
        Err(errors)
    }
}

fn resolve_connections_map<'i>(
    grid: &Grid<'i>,
    labels: &MapId<'i, IndexPos>,
    connections_map: MapPos<Vec<UnresolvedConnection<'i>>>,
) -> Result<Vec<Connection>, MapPos<HashMap<usize, ResolutionError<'i>>>> {
    let mut errors: MapPos<HashMap<usize, ResolutionError>> = HashMap::new();
    let mut res = Vec::new();

    for (from, connections) in connections_map {
        for (i, unresolved) in connections.into_iter().enumerate() {
            match grid.normalize_destination(from, unresolved.to, labels) {
                Ok(to) => res.push(Connection {
                    from: (from, unresolved.sides.0),
                    to: (to, unresolved.sides.1),
                    attrs: unresolved.attrs,
                }),
                Err(resolution_error) => {
                    errors.entry(from).or_default().insert(i, resolution_error);
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(res)
    } else {
        Err(errors)
    }
}

fn resolve_id_map<'i, T: Clone>(grid: &Grid<'i>, map_id: MapId<T>) -> MapPos<T> {
    let mut res = HashMap::new();

    for (id, value) in map_id {
        if let Some(positions) = grid.get_positions(&id) {
            for &pos in positions {
                res.insert(pos, value.clone());
            }
        }
    }

    res
}

trait Overwrite {
    fn overwrite(old: Self, new: Self) -> Self;
}

impl Overwrite for NodeAttributes {
    fn overwrite(old: Self, new: Self) -> Self {
        Self {
            text: new.text.or(old.text),
            class: new.class.or(old.class),
            shape: new.shape.or(old.shape),
        }
    }
}

impl Overwrite for ConnectionAttributes {
    fn overwrite(old: Self, new: Self) -> Self {
        Self {
            text: new.text.or(old.text),
            class: new.class.or(old.class),
        }
    }
}

impl<T> Overwrite for Vec<T> {
    fn overwrite(_old: Self, new: Self) -> Self {
        new
    }
}

impl<T: Overwrite> Overwrite for Option<T> {
    fn overwrite(old: Self, new: Self) -> Self {
        match (old, new) {
            (None, None) => None,
            (Some(x), None) => Some(x),
            (None, Some(x)) => Some(x),
            (Some(old), Some(new)) => Some(Overwrite::overwrite(old, new)),
        }
    }
}

impl<K: Eq + std::hash::Hash, V: Overwrite> Overwrite for HashMap<K, V> {
    fn overwrite(old: Self, new: Self) -> Self {
        let mut res = old;

        for (key, new_val) in new {
            let old_val = res.remove(&key);
            let result = Overwrite::overwrite(old_val, Some(new_val)).unwrap();
            res.insert(key, result);
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::{
        parse::ast::{Direction, Document},
        pos::pos,
        test::{assert_eq, id, map, set},
    };

    use super::{
        super::grid::ResolutionError,
        {Flou, LogicError},
    };

    macro_rules! parse_flou {
        (grid: $grid:literal $(,)?) => {{
            let input = concat!("grid { ", $grid, " }");
            let document = Document::parse(input).unwrap();
            Flou::try_from(document)
        }};

        (grid: $grid:literal, define: $define:literal $(,)?) => {{
            let input = concat!("grid { ", $grid, " } ", "define { ", $define, " }");
            let document = Document::parse(input).unwrap();
            Flou::try_from(document)
        }};
    }

    #[test]
    fn duplicate_labels() {
        let flou = parse_flou! {
            grid: "block#foo; question#bar; block#foo; name#cat, block#bar;",
        };

        assert_eq!(
            flou.unwrap_err(),
            LogicError::DuplicateLabels(map([
                (id("foo"), set([pos(0, 0), pos(0, 2)])),
                (id("bar"), set([pos(0, 1), pos(1, 3)])),
            ]))
        );
    }

    #[test]
    fn duplicate_node_attributes_in_definitions() {
        let flou = parse_flou! {
            grid: "foo; bar;",
            define: r#"
                foo(shape: rect, shape: diamond);
                bar(shape: rect, text: "hello", shape: diamond, connect: n:n@s, text: "hey");
            "#,
        };

        assert_eq!(
            flou.unwrap_err(),
            LogicError::DuplicateNodeAttributesInDefinitions(map([
                (id("foo"), set(["shape"])),
                (id("bar"), set(["shape", "text"])),
            ]))
        );
    }

    #[test]
    fn duplicate_connection_attributes_in_definitions() {
        let flou = parse_flou! {
            grid: "foo; bar;",
            define: r#"
                foo(connect: n:n@s(text: "hi", text: "hello"));
                bar(connect: {n:n@n(text: "hey", class: "hello"); n:n@e(class: "hi", class: "hello")});
            "#,
        };

        assert_eq!(
            flou.unwrap_err(),
            LogicError::DuplicateConnectionAttributesInDefinitions(map([
                (id("foo"), map([(0, set(["text"]))])),
                (id("bar"), map([(1, set(["class"]))])),
            ]))
        );
    }

    #[test]
    fn duplicate_node_attributes_in_grid() {
        let flou = parse_flou! {
            grid: r#"
                foo(shape: rect, text: "hi", shape: diamond);
                bar(connect: n:n@s, shape: circle, text: "hello", shape: rect, connect: n:n#end);
            "#,
        };

        assert_eq!(
            flou.unwrap_err(),
            LogicError::DuplicateNodeAttributesInGrid(map([
                (pos(0, 0), set(["shape"])),
                (pos(0, 1), set(["connect", "shape"])),
            ]))
        );
    }

    #[test]
    fn duplicate_connection_attributes_in_grid() {
        let flou = parse_flou! {
            grid: r#"
                foo(connect: n:n@s(text: "hi", text: "hello"));
                _, bar(connect: {n:n@n(text: "hey", class: "hello"); n:n@e(class: "hi", class: "hello")});
            "#,
        };

        assert_eq!(
            flou.unwrap_err(),
            LogicError::DuplicateConnectionAttributesInGrid(map([
                (pos(0, 0), map([(0, set(["text"]))])),
                (pos(1, 1), map([(1, set(["class"]))])),
            ]))
        );
    }

    #[test]
    fn invalid_destination() {
        let flou = parse_flou! {
            grid: r#"
                start(connect: n:n@n);
                middle;
                end(connect: {n:n#foo; n:n@e});
            "#,
            define: "middle(connect: {n:n@n; n:n@s});",
        };

        assert_eq!(
            flou.unwrap_err(),
            LogicError::InvalidDestination(map([
                (
                    pos(0, 0),
                    map([(0, ResolutionError::InvalidDirection(Direction::North))])
                ),
                (
                    pos(0, 2),
                    map([
                        (0, ResolutionError::UnknownLabel(id("foo"))),
                        (1, ResolutionError::InvalidDirection(Direction::East)),
                    ])
                )
            ]))
        )
    }
}
