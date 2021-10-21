use crate::{
    parse::ast::Direction,
    pos::{pos, PixelPos},
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Viewport {
    pub(crate) origin: PixelPos,
    pub(crate) size: PixelPos,
}

pub(crate) struct Midpoints {
    pub(crate) top: PixelPos,
    pub(crate) bottom: PixelPos,
    pub(crate) left: PixelPos,
    pub(crate) right: PixelPos,
}

impl Midpoints {
    pub(crate) fn get_from_direction(&self, dir: Direction) -> PixelPos {
        match dir {
            Direction::North => self.top,
            Direction::East => self.right,
            Direction::West => self.left,
            Direction::South => self.bottom,
        }
    }
}

impl Viewport {
    pub(crate) fn new(origin: PixelPos, size: PixelPos) -> Self {
        Self { origin, size }
    }

    pub(crate) fn center(&self) -> PixelPos {
        self.origin + self.size / 2
    }

    pub(crate) fn midpoints(&self) -> Midpoints {
        let rel = self.midpoints_relative();

        Midpoints {
            top: self.origin + rel.top,
            bottom: self.origin + rel.bottom,
            left: self.origin + rel.left,
            right: self.origin + rel.right,
        }
    }

    pub(crate) fn midpoints_relative(&self) -> Midpoints {
        let half = self.size / 2;

        Midpoints {
            top: pos(half.x, 0),
            bottom: pos(half.x, self.size.y),
            left: pos(0, half.y),
            right: pos(self.size.x, half.y),
        }
    }
}
