use std::{
    collections::{hash_map, HashMap, HashSet},
    convert::TryFrom,
};

use crate::{
    grid::{Grid, ResolutionError},
    parse::ast::{self, ConnectionDescriptor},
    pos::IndexPos,
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LogicError<'i> {
    /// A label was used more than once.
    DuplicateLabels(MapId<'i, HashSet<IndexPos>>),

    /// There is more than one definition for one identifier.
    DuplicateDefinitions(HashSet<ast::Identifier<'i>>),

    /// Some definitions contain duplicate node attributes.
    DuplicateNodeAttributesInDefinitions(MapId<'i, HashSet<&'static str>>),

    /// Some nodes inside the grid have duplicate node attributes.
    DuplicateNodeAttributesInGrid(MapPos<HashSet<&'static str>>),

    /// Some connections inside the `define` block contain duplicate attributes.
    DuplicateConnectionAttributesInDefinitions(MapId<'i, HashMap<usize, HashSet<&'static str>>>),

    /// Some connections inside the `grid` block contain duplicate attributes.
    DuplicateConnectionAttributesInGrid(MapPos<HashMap<usize, HashSet<&'static str>>>),

    /// One or more connections have destinations that couldn't be resolved.
    InvalidDestination(MapPos<HashMap<usize, ResolutionError<'i>>>),
}

#[derive(Debug, Default, Clone)]
pub(crate) struct NodeAttributes {
    text: Option<String>,
    class: Option<String>,
    shape: Option<ast::NodeShape>,
}

/// Tries to assemble `NodeAttributes` from the vector of individual attributes.
/// The `connect` attribute is separated from the rest so that an independent
/// vector of connections can be created later down the line.
fn parse_node_attributes<'i>(
    attributes: Vec<ast::NodeAttribute<'i>>,
) -> Result<(NodeAttributes, Option<Vec<ast::ConnectionDescriptor<'i>>>), HashSet<&'static str>> {
    let mut res = NodeAttributes::default();
    let mut duplicates = HashSet::new();
    let mut conn_descriptors = None;

    for attribute in attributes {
        match attribute {
            ast::NodeAttribute::Text(text) if res.text.is_none() => res.text = Some(text),
            ast::NodeAttribute::Class(class) if res.class.is_none() => res.class = Some(class),
            ast::NodeAttribute::Shape(shape) if res.shape.is_none() => res.shape = Some(shape),
            ast::NodeAttribute::Connect(descriptors) if conn_descriptors.is_none() => {
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

#[derive(Debug, Default, Clone)]
pub(crate) struct ConnectionAttributes {
    text: Option<String>,
    class: Option<String>,
}

impl TryFrom<Vec<ast::ConnectionAttribute>> for ConnectionAttributes {
    type Error = HashSet<&'static str>;

    fn try_from(attributes: Vec<ast::ConnectionAttribute>) -> Result<Self, Self::Error> {
        let mut res = Self::default();
        let mut duplicates = HashSet::new();

        for attribute in attributes {
            match attribute {
                ast::ConnectionAttribute::Text(text) if res.text.is_none() => res.text = Some(text),
                ast::ConnectionAttribute::Class(class) if res.class.is_none() => {
                    res.class = Some(class)
                }
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
    grid: &ast::Grid<'i>,
) -> Result<MapId<'i, IndexPos>, MapId<'i, HashSet<IndexPos>>> {
    let mut positions: HashMap<ast::Identifier, HashSet<IndexPos>> = HashMap::new();

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

type MapPos<T> = HashMap<IndexPos, T>;
type MapId<'i, T> = HashMap<ast::Identifier<'i>, T>;
type TwoMapId<'i, T1, T2> = (MapId<'i, T1>, MapId<'i, T2>);
type TwoMapPos<T1, T2> = (MapPos<T1>, MapPos<T2>);

fn get_attributes_from_definitions<'i>(
    definitions: MapId<'i, Vec<ast::NodeAttribute<'i>>>,
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
    grid: &ast::Grid<'i>,
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
    definitions: Vec<(ast::Identifier<'i>, Vec<ast::NodeAttribute<'i>>)>,
) -> Result<MapId<Vec<ast::NodeAttribute<'i>>>, HashSet<ast::Identifier<'i>>> {
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

type MapToIncompleteConnection<'i, T> =
    HashMap<T, Vec<(ast::Destination<'i>, ConnectionAttributes)>>;

type MapToDuplicateAttrs<'i, T> = HashMap<T, HashMap<usize, HashSet<&'static str>>>;

fn parse_connection_desc_map<T: Eq + std::hash::Hash + Copy>(
    def_connection_desc_map: HashMap<T, Vec<ConnectionDescriptor>>,
) -> Result<MapToIncompleteConnection<T>, MapToDuplicateAttrs<T>> {
    let mut errors = HashMap::new();
    let mut res = HashMap::new();

    for (id, descriptors) in def_connection_desc_map {
        let mut value = Vec::new();
        for (i, descriptor) in descriptors.into_iter().enumerate() {
            match ConnectionAttributes::try_from(descriptor.attrs) {
                Ok(attrs) => {
                    value.push((descriptor.to, attrs));
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
    connections_map: MapPos<Vec<(ast::Destination<'i>, ConnectionAttributes)>>,
) -> Result<Vec<Connection>, MapPos<HashMap<usize, ResolutionError<'i>>>> {
    let mut errors: MapPos<HashMap<usize, ResolutionError>> = HashMap::new();
    let mut res = Vec::new();

    for (from, connections) in connections_map {
        for (i, (destination, attrs)) in connections.into_iter().enumerate() {
            match grid.normalize_destination(from, destination, labels) {
                Ok(to) => res.push(Connection {
                    from,
                    to: to.into(),
                    attrs,
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
            for pos in positions {
                res.insert(pos, value.clone());
            }
        }
    }

    res
}

#[derive(Debug)]
struct Connection {
    from: IndexPos,
    to: IndexPos,
    attrs: ConnectionAttributes,
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

#[derive(Debug)]
struct Flou<'i> {
    grid: Grid<'i>,
    connections: Vec<Connection>,
    node_attributes: MapPos<NodeAttributes>,
}

impl<'i> TryFrom<ast::Document<'i>> for Flou<'i> {
    type Error = LogicError<'i>;

    fn try_from(document: ast::Document<'i>) -> Result<Self, Self::Error> {
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

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::{
        flou::{Flou, LogicError},
        parse::ast,
        pos::pos,
        test::{id, map, set},
    };

    #[test]
    fn duplicate_labels() {
        let input = r#"
        grid {
            block#foo;
            question#bar;
            block#foo;
            name#cat, block#bar;
        }
        "#;

        let document = ast::Document::parse(input).unwrap();

        let flou = Flou::try_from(document);

        assert_eq!(
            flou.unwrap_err(),
            LogicError::DuplicateLabels(map! {
                id("foo") => set!{ pos(0, 0), pos(0, 2) },
                id("bar") => set!{ pos(0, 1), pos(1, 3)},
            })
        );
    }
}
