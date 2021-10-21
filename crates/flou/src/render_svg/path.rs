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

    fn straight_line(from: Self, to: Self) -> Option<Direction> {
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

    fn grid_aligned(&self) -> bool {
        self.grid_x_aligned() && self.grid_y_aligned()
    }

    fn grid_x_aligned(&self) -> bool {
        self.x & 1 == 1
    }

    fn grid_y_aligned(&self) -> bool {
        self.y & 1 == 1
    }
}

type OriginSide = (IndexPos, Direction);
type OriginOptSide = (IndexPos, Option<Direction>);

fn resolve_sides(
    (a, a_side): OriginOptSide,
    (b, b_side): OriginOptSide,
) -> (OriginSide, OriginSide) {
    fn get_best_side(from: IndexPos, to: PaddedPos) -> Direction {
        let sides = vec![
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ];
        let from: PaddedPos = from.into();

        sides
            .into_iter()
            .max_by(|&a, &b| {
                let a = from + PaddedPos::from(a);
                let b = from + PaddedPos::from(b);
                let dist_a = PaddedPos::taxicab(a, to);
                let dist_b = PaddedPos::taxicab(b, to);
                dist_a
                    .cmp(&dist_b)
                    .then_with(|| get_best_corner(a, to).1.cmp(&get_best_corner(b, to).1))
            })
            .unwrap()
    }

    match (a_side, b_side) {
        (None, None) => {
            let b_side = Direction::North;
            let a_side = get_best_side(a, PaddedPos::from(b) + PaddedPos::from(b_side));
            ((a, a_side), (b, b_side))
        }
        (None, Some(b_side)) => {
            let a_side = get_best_side(a, PaddedPos::from(b) + PaddedPos::from(b_side));
            ((a, a_side), (b, b_side))
        }
        (Some(a_side), None) => {
            let b_side = get_best_side(b, PaddedPos::from(a) + PaddedPos::from(a_side));
            ((a, a_side), (b, b_side))
        }
        (Some(a_side), Some(b_side)) => ((a, a_side), (b, b_side)),
    }
}

fn get_best_corner(a: PaddedPos, b: PaddedPos) -> (PaddedPos, FreeAxisCount) {
    let corners: (PaddedPos, PaddedPos) = (pos(a.x, b.y), pos(b.x, a.y));
    let corners = (
        (corners.0, FreeAxisCount::from_pos(corners.0)),
        (corners.1, FreeAxisCount::from_pos(corners.1)),
    );

    std::cmp::max_by_key(corners.0, corners.1, |&(_, lane_count)| lane_count)
}

fn path_with_sides(
    grid: &Grid,
    (from, from_side): OriginSide,
    (to, to_side): OriginSide,
) -> Vec<PaddedPos> {
    if PaddedPos::PADDING != 1 {
        panic!("Algorithm is designed to work with a padding of 1");
    }

    if from == to {
        return vec![];
    }

    let s_from = PaddedPos::from(from) + PaddedPos::from(from_side);
    let s_to = PaddedPos::from(to) + PaddedPos::from(to_side);

    if s_from == s_to {
        return vec![from.into(), to.into()];
    }

    if let Some(s_dir) = PaddedPos::straight_line(s_from, s_to) {
        if let Some(dir) = IndexPos::straight_line(from, to) {
            if s_dir == dir {
                if let Some(dest) = grid.walk(from, dir.into()) {
                    if dest == to {
                        return vec![from.into(), to.into()];
                    }
                }
            }
        }
    }

    let (corner, lane_count) = get_best_corner(s_from, s_to);
    let connect_to_corner = |corner| vec![from.into(), s_from, corner, s_to, to.into()];

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

pub(crate) fn path(grid: &Grid, from: OriginOptSide, to: OriginOptSide) -> Vec<PaddedPos> {
    let (from, to) = resolve_sides(from, to);
    path_with_sides(grid, from, to)
}
