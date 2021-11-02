use std::collections::HashMap;

use crate::{
    parse::ast::{Destination, Direction, Grid as ASTGrid, Identifier},
    pos::IndexPos,
};

#[derive(Debug, PartialEq, Eq)]
pub enum ResolutionError<'i> {
    InvalidDirection(Direction),
    UnknownLabel(Identifier<'i>),
}

#[derive(Debug)]
pub(crate) struct Grid<'i> {
    pub(crate) size: IndexPos,
    pub(crate) position_to_id: HashMap<IndexPos, Identifier<'i>>,
    id_to_positions: HashMap<Identifier<'i>, Vec<IndexPos>>,
}

impl<'i> Grid<'i> {
    pub(crate) fn normalize_destination(
        &self,
        from: IndexPos,
        to: Destination<'i>,
        labels: &HashMap<Identifier<'i>, IndexPos>,
    ) -> Result<IndexPos, ResolutionError<'i>> {
        match to {
            Destination::Itself => Ok(from),
            Destination::Relative(dir) => {
                let step = IndexPos::from(dir);
                self.walk(from, step)
                    .ok_or(ResolutionError::InvalidDirection(dir))
            }
            Destination::Label(label) => labels
                .get(&label)
                .copied()
                .ok_or(ResolutionError::UnknownLabel(label)),
        }
    }

    pub(crate) fn get_positions(&self, id: &Identifier<'i>) -> Option<&Vec<IndexPos>> {
        self.id_to_positions.get(id)
    }

    pub(crate) fn walk(&self, start: IndexPos, step: IndexPos) -> Option<IndexPos> {
        let mut current = start;
        loop {
            current += step;

            break match self.get_id(current) {
                None => None,                   // Out of bounds
                Some(Some(_)) => Some(current), // Ran into something; stop immediately
                Some(None) => continue,         // Empty space; keep moving
            };
        }
    }

    fn get_id(&self, pos: IndexPos) -> Option<Option<&Identifier>> {
        pos.in_bounds(self.size)
            .then(|| self.position_to_id.get(&pos))
    }
}

impl<'i> From<&ASTGrid<'i>> for Grid<'i> {
    fn from(grid: &ASTGrid<'i>) -> Self {
        let mut position_to_id = HashMap::new();
        let mut id_to_positions: HashMap<Identifier, Vec<_>> = HashMap::new();

        for (pos, node) in grid.nodes() {
            position_to_id.insert(pos, node.id);
            id_to_positions.entry(node.id).or_default().push(pos);
        }

        Self {
            size: grid.size(),
            position_to_id,
            id_to_positions,
        }
    }
}
