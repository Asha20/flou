use num_traits::{Num, Signed};

use std::{fmt, marker::PhantomData, ops};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct IndexSpace;
pub type IndexPos = Position2D<isize, IndexSpace>;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct PixelSpace;
pub type PixelPos = Position2D<i32, PixelSpace>;

impl_pos_from!(Position2D<usize, IndexSpace>, IndexPos, isize);
impl_pos_from!(PixelPos, IndexPos, isize);
impl_pos_from!(IndexPos, PixelPos, i32);

#[derive(PartialEq, Eq, std::hash::Hash)]
pub struct Position2D<T: Num, U> {
    pub x: T,
    pub y: T,
    #[doc(hidden)]
    _unit: PhantomData<U>,
}

pub(crate) fn pos<T: Num, U>(x: T, y: T) -> Position2D<T, U> {
    Position2D::new(x, y)
}

impl<T: Num + Copy, U> Copy for Position2D<T, U> {}

impl<T: Num + Clone, U> Clone for Position2D<T, U> {
    fn clone(&self) -> Self {
        Self {
            x: self.x.clone(),
            y: self.y.clone(),
            _unit: PhantomData,
        }
    }
}

impl<T: Num + fmt::Debug, U> fmt::Debug for Position2D<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pos({:?}, {:?})", self.x, self.y)
    }
}

impl<T: Num, U> Position2D<T, U> {
    pub(crate) fn new(x: T, y: T) -> Self {
        Self {
            x,
            y,
            _unit: PhantomData,
        }
    }
}

impl<T: Num + Signed, U> ops::Neg for Position2D<T, U> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl<T: Num + Copy, U> From<T> for Position2D<T, U> {
    fn from(val: T) -> Self {
        Self::new(val, val)
    }
}

impl<T: Num, U, I: Into<Self>> ops::Mul<I> for Position2D<T, U> {
    type Output = Self;

    fn mul(self, rhs: I) -> Self::Output {
        let rhs = rhs.into();
        Self::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl<T: Num + Copy, U, I: Into<Self>> ops::Div<I> for Position2D<T, U> {
    type Output = Self;

    fn div(self, rhs: I) -> Self::Output {
        let rhs = rhs.into();
        Self::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl<T: Num + ops::AddAssign, U, I: Into<Self>> ops::AddAssign<I> for Position2D<T, U> {
    fn add_assign(&mut self, rhs: I) {
        let rhs = rhs.into();
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T: Num + ops::SubAssign, U, I: Into<Self>> ops::SubAssign<I> for Position2D<T, U> {
    fn sub_assign(&mut self, rhs: I) {
        let rhs = rhs.into();
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T: Num + Copy, U, I: Into<Self>> ops::Add<I> for Position2D<T, U> {
    type Output = Self;

    fn add(self, rhs: I) -> Self::Output {
        let rhs = rhs.into();
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<T: Num + Copy, U, I: Into<Self>> ops::Sub<I> for Position2D<T, U> {
    type Output = Self;

    fn sub(self, rhs: I) -> Self::Output {
        let rhs = rhs.into();
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<T: Num + Ord, U> Position2D<T, U> {
    pub(crate) fn in_bounds(&self, bounds: Self) -> bool {
        self.x >= T::zero() && self.x < bounds.x && self.y >= T::zero() && self.y < bounds.y
    }
}

macro_rules! impl_pos_from {
    ($from:ty, $to:ty) => {
        impl From<$from> for $to {
            fn from(other: $from) -> Self {
                Self::new(other.x, other.y)
            }
        }
    };

    ($from:ty, $to:ty, $cast:ty) => {
        impl From<$from> for $to {
            fn from(other: $from) -> Self {
                Self::new(other.x as $cast, other.y as $cast)
            }
        }
    };
}

pub(crate) use impl_pos_from;

use crate::parse::ast::Direction;

impl<T: Num + Signed, U> From<Direction> for Position2D<T, U> {
    fn from(dir: Direction) -> Self {
        let one = T::one();
        let zero = T::zero();
        match dir {
            Direction::North => Self::new(zero, -one),
            Direction::South => Self::new(zero, one),
            Direction::West => Self::new(-one, zero),
            Direction::East => Self::new(one, zero),
        }
    }
}
