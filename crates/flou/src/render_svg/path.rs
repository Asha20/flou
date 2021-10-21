#![allow(dead_code)]

use std::{cmp::Ordering, ops::Sub};

use num_traits::Num;

use crate::{
    parse::ast::Direction,
    parts::Grid,
    pos::{pos, IndexPos, Position2D},
    render_svg::renderer::PaddedPos,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Axis {
    X,
    Y,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
            (false, true) => Self::One(Axis::X),
            (true, false) => Self::One(Axis::Y),
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
        match distance.x.cmp(&T::zero()) {
            Ordering::Less => Some(Direction::West),
            Ordering::Greater => Some(Direction::East),
            Ordering::Equal => match distance.y.cmp(&T::zero()) {
                Ordering::Less => Some(Direction::North),
                Ordering::Greater => Some(Direction::South),
                Ordering::Equal => None,
            },
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

fn resolve_sides(
    grid: &Grid,
    (a, a_side): (IndexPos, Option<Direction>),
    (b, b_side): (IndexPos, Option<Direction>),
) -> (PosSide, PosSide) {
    fn get_best_side(grid: &Grid, from: IndexPos, to: PosSide) -> Direction {
        let sides = vec![
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ];

        sides
            .into_iter()
            .max_by(|&a_side, &b_side| {
                let a = PosSide::new(from, a_side);
                let b = PosSide::new(from, b_side);

                let a_line = can_draw_straight_line(grid, a, to);
                let b_line = can_draw_straight_line(grid, b, to);

                a_line
                    .cmp(&b_line)
                    .then_with(|| {
                        let dist_a = PaddedPos::taxicab(a.into(), to.into());
                        let dist_b = PaddedPos::taxicab(b.into(), to.into());
                        dist_a.cmp(&dist_b)
                    })
                    .then_with(|| get_best_corner(a, to).1.cmp(&get_best_corner(b, to).1))
            })
            .unwrap()
    }

    match (a_side, b_side) {
        (None, None) => {
            if a == b {
                return (
                    PosSide::new(a, Direction::West),
                    PosSide::new(b, Direction::North),
                );
            }

            let b_side = Direction::North;
            let a_side = get_best_side(grid, a, PosSide::new(b, b_side));
            (PosSide::new(a, a_side), PosSide::new(b, b_side))
        }
        (None, Some(b_side)) => {
            let a_side = get_best_side(grid, a, PosSide::new(b, b_side));
            (PosSide::new(a, a_side), PosSide::new(b, b_side))
        }
        (Some(a_side), None) => {
            let b_side = get_best_side(grid, b, PosSide::new(a, a_side));
            (PosSide::new(a, a_side), PosSide::new(b, b_side))
        }
        (Some(a_side), Some(b_side)) => (PosSide::new(a, a_side), PosSide::new(b, b_side)),
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
    if let Some(s_dir) = PaddedPos::straight_line(from.into(), to.into()) {
        if let Some(dir) = IndexPos::straight_line(from.origin, to.origin) {
            if s_dir == dir {
                if let Some(dest) = grid.walk(from.origin, dir.into()) {
                    if dest == to.origin {
                        return true;
                    }
                }
            }
        }
    }

    false
}

fn path_with_sides(grid: &Grid, from: PosSide, to: PosSide) -> Vec<PaddedPos> {
    if PaddedPos::PADDING != 1 {
        panic!("Algorithm is designed to work with a padding of 1");
    }

    let s_from: PaddedPos = from.into();
    let s_to: PaddedPos = to.into();

    if s_from == s_to {
        return vec![from.into(), to.into()];
    }

    if can_draw_straight_line(grid, from, to) {
        return vec![from.origin.into(), to.origin.into()];
    }

    if from.origin == to.origin {
        // When this is implemented, it will handle the case where from_side and to_side
        // are opposites. There should never be a case where they are equal; this should
        // be prevented earlier, as a parsing error.
        unimplemented!()
    }

    let (corner, lane_count) = get_best_corner(s_from, s_to);
    let connect_to_corner =
        |corner| vec![from.origin.into(), s_from, corner, s_to, to.origin.into()];

    match lane_count {
        FreeAxisCount::Two => connect_to_corner(corner),
        FreeAxisCount::One(free_axis) => {
            let dirs = match free_axis {
                Axis::X => (Direction::West, Direction::East),
                Axis::Y => (Direction::North, Direction::South),
            };
            let corner_candidates = (
                corner + PaddedPos::from(dirs.0),
                corner + PaddedPos::from(dirs.1),
            );
            let corner = std::cmp::min_by_key(corner_candidates.0, corner_candidates.1, |&c| {
                PaddedPos::taxicab(s_from, c) + PaddedPos::taxicab(s_to, c)
            });

            connect_to_corner(corner)
        },
        FreeAxisCount::Zero => unreachable!("The FreeAxisCount of the two calculated corners can either be (0, 2) or (1, 1), so by taking the max we should never end up here."),
    }
}

type OriginOptSide = (IndexPos, Option<Direction>);
pub(crate) fn get_path(grid: &Grid, from: OriginOptSide, to: OriginOptSide) -> Vec<PaddedPos> {
    let (from, to) = resolve_sides(grid, from, to);
    path_with_sides(grid, from, to)
}
