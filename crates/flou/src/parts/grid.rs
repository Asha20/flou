use std::collections::HashMap;

use crate::{
    parse::ast::{Destination, Direction, Grid as ASTGrid, Identifier},
    pos::{IndexPos, Position2D},
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct PaddedSpace;
pub(crate) type PaddedPos = Position2D<isize, PaddedSpace>;

impl PaddedPos {
    const PADDING: isize = 1;
}

impl From<IndexPos> for PaddedPos {
    fn from(other: IndexPos) -> Self {
        let res = other * (PaddedPos::PADDING + 1) + PaddedPos::PADDING;
        Self::new(res.x, res.y)
    }
}

impl From<PaddedPos> for IndexPos {
    fn from(other: PaddedPos) -> Self {
        let res = (other - PaddedPos::PADDING) / (PaddedPos::PADDING + 1);
        Self::new(res.x, res.y)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ResolutionError<'i> {
    InvalidDirection(Direction),
    UnknownLabel(Identifier<'i>),
}

#[derive(Debug)]
pub(crate) struct Grid<'i> {
    size: IndexPos,
    position_to_id: HashMap<PaddedPos, Identifier<'i>>,
    id_to_positions: HashMap<Identifier<'i>, Vec<PaddedPos>>,
}

impl<'i> Grid<'i> {
    pub(crate) fn normalize_destination(
        &self,
        from: IndexPos,
        to: Destination<'i>,
        labels: &HashMap<Identifier<'i>, IndexPos>,
    ) -> Result<PaddedPos, ResolutionError<'i>> {
        match to {
            Destination::Relative(dir) => {
                let step = PaddedPos::from(dir);
                self.walk(from.into(), step)
                    .ok_or(ResolutionError::InvalidDirection(dir))
            }
            Destination::Label(label) => labels
                .get(&label)
                .map(|&pos| pos.into())
                .ok_or(ResolutionError::UnknownLabel(label)),
        }
    }

    pub(crate) fn get_positions(&self, id: &Identifier<'i>) -> Option<Vec<IndexPos>> {
        self.id_to_positions
            .get(id)
            .map(|positions| positions.iter().map(|&x| x.into()).collect())
    }

    fn get_id(&self, pos: PaddedPos) -> Option<Option<&Identifier>> {
        pos.in_bounds(self.size.into())
            .then(|| self.position_to_id.get(&pos))
    }

    fn walk(&self, start: PaddedPos, step: PaddedPos) -> Option<PaddedPos> {
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
}

impl<'i> From<&ASTGrid<'i>> for Grid<'i> {
    fn from(grid: &ASTGrid<'i>) -> Self {
        let mut position_to_id = HashMap::new();
        let mut id_to_positions: HashMap<Identifier, Vec<_>> = HashMap::new();

        for (pos, node) in grid.nodes() {
            let pos: PaddedPos = pos.into();

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
