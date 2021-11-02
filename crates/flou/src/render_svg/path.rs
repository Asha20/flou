#![allow(dead_code)]

use std::{cmp::Ordering, ops::Sub};

use num_traits::Num;

use crate::{
    parse::ast::Direction,
    parts::Grid,
    pos::{pos, IndexPos, Position2D},
    render_svg::renderer::PaddedPos,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Axis {
    X,
    Y,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum FreeAxisCount {
    Zero,
    One(Axis),
    Two,
}

impl FreeAxisCount {
    fn from_pos(pos: PaddedPos) -> Self {
        let x_aligned = pos.x & 1 == 1;
        let y_aligned = pos.y & 1 == 1;
        match (x_aligned, y_aligned) {
            (false, false) => Self::Two,
            (false, true) => Self::One(Axis::Y),
            (true, false) => Self::One(Axis::X),
            (true, true) => Self::Zero,
        }
    }
}

impl<T: Num + Ord + Copy, U> Position2D<T, U>
where
    Self: Sub<Output = Self>,
{
    fn x_direction(from: Self, to: Self) -> Option<Direction> {
        match from.x.cmp(&to.x) {
            Ordering::Less => Some(Direction::East),
            Ordering::Greater => Some(Direction::West),
            Ordering::Equal => None,
        }
    }

    fn y_direction(from: Self, to: Self) -> Option<Direction> {
        match from.y.cmp(&to.y) {
            Ordering::Less => Some(Direction::South),
            Ordering::Greater => Some(Direction::North),
            Ordering::Equal => None,
        }
    }

    pub(crate) fn straight_line(from: Self, to: Self) -> Option<Direction> {
        let distance = to - from;

        match (distance.x == T::zero(), distance.y == T::zero()) {
            (false, false) | (true, true) => None,
            (false, true) => Self::x_direction(from, to),
            (true, false) => Self::y_direction(from, to),
        }
    }

    fn taxicab(from: Self, to: Self) -> T {
        Self::x_distance(from, to) + Self::y_distance(from, to)
    }

    fn x_distance(from: Self, to: Self) -> T {
        match from.x > to.x {
            true => from.x - to.x,
            false => to.x - from.x,
        }
    }

    fn y_distance(from: Self, to: Self) -> T {
        match from.y > to.y {
            true => from.y - to.y,
            false => to.y - from.y,
        }
    }
}

impl PaddedPos {
    pub(crate) fn snap_to_grid(&self) -> Self {
        let x = self.x - ((self.x & 1) ^ 1);
        let y = self.y - ((self.y & 1) ^ 1);
        Self::new(x, y)
    }

    pub(crate) fn grid_aligned(&self) -> bool {
        self.grid_x_aligned() && self.grid_y_aligned()
    }

    pub(crate) fn grid_x_aligned(&self) -> bool {
        self.x & 1 == 1
    }

    pub(crate) fn grid_y_aligned(&self) -> bool {
        self.y & 1 == 1
    }
}

#[derive(Debug, Clone, Copy)]
struct PosSide {
    origin: IndexPos,
    side: Direction,
}

impl PosSide {
    fn new(origin: IndexPos, side: Direction) -> Self {
        Self { origin, side }
    }
}

impl From<PosSide> for PaddedPos {
    fn from(x: PosSide) -> Self {
        Self::from(x.origin) + Self::from(x.side)
    }
}

fn get_best_corner<I: Into<PaddedPos>>(a: I, b: I) -> (PaddedPos, FreeAxisCount) {
    let a = a.into();
    let b = b.into();
    let corners: (PaddedPos, PaddedPos) = (pos(a.x, b.y), pos(b.x, a.y));
    let corners = (
        (corners.0, FreeAxisCount::from_pos(corners.0)),
        (corners.1, FreeAxisCount::from_pos(corners.1)),
    );

    std::cmp::max_by_key(corners.0, corners.1, |&(_, lane_count)| lane_count)
}

fn can_draw_straight_line(grid: &Grid, from: PosSide, to: PosSide) -> bool {
    if let Some(dir) = PaddedPos::straight_line(from.into(), to.into()) {
        if !PaddedPos::from(from).grid_aligned() {
            return true;
        }

        if IndexPos::straight_line(from.origin, to.origin).is_some() {
            if let Some(dest) = grid.walk(from.origin, dir.into()) {
                if dest == to.origin {
                    return true;
                }
            }
        }
    }

    false
}

pub(crate) fn get_path(
    grid: &Grid,
    from: (IndexPos, Direction),
    to: (IndexPos, Direction),
) -> Vec<PaddedPos> {
    if PaddedPos::PADDING != 1 {
        panic!("Algorithm is designed to work with a padding of 1");
    }

    let from = PosSide::new(from.0, from.1);
    let to = PosSide::new(to.0, to.1);

    // TODO: Improve algorithm when from and to lie on the same line but can't
    // be connected directly.

    let s_from: PaddedPos = from.into();
    let s_to: PaddedPos = to.into();

    if s_from == s_to {
        return vec![from.origin.into(), to.origin.into()];
    }

    // TODO: Don't insert s_from and s_to if they lie on the line
    // that the first and last point make.
    if can_draw_straight_line(grid, from, to) {
        return vec![from.origin.into(), s_from, s_to, to.origin.into()];
    }

    if from.origin == to.origin {
        // When this is implemented, it will handle the case where from_side and to_side
        // are opposites. There should never be a case where they are equal; this should
        // be prevented earlier, as a parsing error.
        unimplemented!()
    }

    let (corner, lane_count) = get_best_corner(s_from, s_to);
    let make_connection = |mid: Vec<PaddedPos>| {
        let mut res = vec![from.origin.into(), s_from];
        res.extend(mid);
        res.push(s_to);
        res.push(to.origin.into());
        res
    };

    match lane_count {
        FreeAxisCount::Two => make_connection(vec![corner]),
        FreeAxisCount::One(free_axis) => {
            let dirs = match free_axis {
                Axis::X => (Direction::West, Direction::East),
                Axis::Y => (Direction::North, Direction::South),
            };
            let corner_candidates = (
                (dirs.0, corner + PaddedPos::from(dirs.0)),
                (dirs.1, corner + PaddedPos::from(dirs.1)),
            );
            let (dir, corner) = std::cmp::min_by_key(corner_candidates.0, corner_candidates.1, |&(_, c)| {
                PaddedPos::taxicab(s_from, c) + PaddedPos::taxicab(s_to, c)
            });

            let mut res = vec![];
            if PaddedPos::straight_line(s_from, corner).is_none() {
                res.push(s_from + PaddedPos::from(dir));
            }
            res.push(corner);
            if PaddedPos::straight_line(s_to, corner).is_none() {
                res.push(s_to + PaddedPos::from(dir));
            }

            make_connection(res)
        },
        FreeAxisCount::Zero => unreachable!("The FreeAxisCount of the two calculated corners can either be (0, 2) or (1, 1), so by taking the max we should never end up here."),
    }
}
