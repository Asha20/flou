use std::collections::{HashMap, HashSet};

use crate::{parse::ast::Identifier, pos::IndexPos};

use super::grid::ResolutionError;

type MapPos<T> = HashMap<IndexPos, T>;
type MapId<'i, T> = HashMap<Identifier<'i>, T>;

#[derive(Debug, PartialEq, Eq)]
pub enum LogicError<'i> {
    /// A label was used more than once.
    DuplicateLabels(MapId<'i, HashSet<IndexPos>>),

    /// There is more than one definition for one identifier.
    DuplicateDefinitions(HashSet<Identifier<'i>>),

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
