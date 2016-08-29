A `Direction` is a simple compass direction. There are only
eight of them because we only handle the eight directions
that are immediately expressible via a grid:
```
\|/
- -
/|\
```

/// A `Direction` is a simple compass direction on the grid.
```rust
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub enum Direction { N, NE, E, SE, S, SW, W, NW }

pub const DIRECTIONS: &'static [Direction] =
    &[Direction::N, Direction::NE, Direction::E, Direction::SE,
      Direction::S, Direction::SW, Direction::W, Direction::NW];


/// A turn chooses between clockwise (`CW`) and counter-clockwise (`CCW`).
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Turn {
    /// Clockwise
    CW,
    /// Counter-clockwise
    CCW,
}

impl Turn {
    pub fn reverse(&self) -> Self {
        use self::Turn::*;
        match *self {
            CW => CCW,
            CCW => CW,
        }
    }
}

impl Direction {
    pub fn ver_north(&self) -> i32 {
        use directions::Direction::*;
        match *self {
            N | NE | NW => 1,
            S | SE | SW => -1,
            E | W => 0,
        }
    }

    pub fn ver_south(&self) -> i32 { -self.ver_north() }

    pub fn hor_east(&self) -> i32 {
        use directions::Direction::*;
        match *self {
            E | NE | SE => 1,
            W | NW | SW => -1,
            N | S => 0,
        }
    }

    pub fn hor_west(&self) -> i32 { -self.hor_east() }

    pub fn reverse(&self) -> Self {
        use self::Direction::*;
        match *self {
            N => S, NE => SW, E => W, SE => NW,
            S => N, SW => NE, W => E, NW => SE,
        }
    }

    pub fn veer(&self, t: Turn) -> Self {
        use self::Direction::*;
        use self::Turn::*;
        match (*self, t) {
            (N, CW) => NE, (NE, CW) => E, (E, CW) => SE, (SE, CW) => S,
            (S, CW) => SW, (SW, CW) => W, (W, CW) => NW, (NW, CW) => N,
            (N, CCW) => NW, (NW, CCW) => W, (W, CCW) => SW, (SW, CCW) => S,
            (S, CCW) => SE, (SE, CCW) => E, (E, CCW) => NE, (NE, CCW) => N,
        }
    }

    pub fn sharp_turn(&self, t: Turn) -> Self {
        // a sharp turn ends up being the same as reversing and then
        // veering to the reverse direction (because reversing is like
        // taking the sharpest turn one notch too far).
        self.reverse().veer(t.reverse())
    }
}

mod dir_tests {
    #[test]
    fn dir_basics() {
        use directions::Direction::*;
        assert_eq!(NE.ver_north(), 1);
        assert_eq!(NW.ver_south(), -1);
        assert_eq!(S.hor_east(), 0);
        assert_eq!(NE.hor_east(), 1);
        assert_eq!(SW.hor_west(), 1);
    }
}

pub trait ToDirections { fn to_directions(&self) -> Vec<Direction>; }

pub struct N;
pub struct NE;
pub struct E;
pub struct SE;
pub struct S;
pub struct SW;
pub struct W;
pub struct NW;

pub struct N_;
pub struct _E;
pub struct S_;
pub struct _W;

pub struct Horizontal;
pub struct Vertical;
pub struct NonHorizontal;
pub struct NonVertical;
pub struct NonNorth;
pub struct NonSouth;
pub struct NonEast;
pub struct NonWest;
pub struct Any;

impl ToDirections for Direction {
    fn to_directions(&self) -> Vec<Direction> { vec![*self] }
}

impl ToDirections for Horizontal {
    fn to_directions(&self) -> Vec<Direction> { vec![Direction::W, Direction::E] }
}

impl ToDirections for Vertical {
    fn to_directions(&self) -> Vec<Direction> { vec![Direction::N, Direction::S] }
}

impl ToDirections for NonHorizontal {
    fn to_directions(&self) -> Vec<Direction> {
        use directions::Direction::*;
        vec![N, NE, NW, S, SE, SW]
    }
}

impl ToDirections for NonVertical {
    fn to_directions(&self) -> Vec<Direction> {
        use directions::Direction::*;
        vec![E, NE, SE, W, NW, SW]
    }
}

impl ToDirections for NonNorth {
    fn to_directions(&self) -> Vec<Direction> {
        use directions::Direction::*;
        vec![E, SE, S, SW, W]
    }
}

impl ToDirections for NonSouth {
    fn to_directions(&self) -> Vec<Direction> {
        use directions::Direction::*;
        vec![E, NE, N, NW, W]
    }
}

impl ToDirections for NonEast {
    fn to_directions(&self) -> Vec<Direction> {
        use directions::Direction::*;
        vec![N, NW, W, SW, S]
    }
}

impl ToDirections for NonWest {
    fn to_directions(&self) -> Vec<Direction> {
        use directions::Direction::*;
        vec![N, NE, E, SE, S]
    }
}

impl ToDirections for Any {
    fn to_directions(&self) -> Vec<Direction> { DIRECTIONS.iter().cloned().collect() }
}

macro_rules! to_dirs {
    ($I:ident) => {
        impl ToDirections for $I {
            fn to_directions(&self) -> Vec<Direction> {
                vec![Direction::$I]
            }
        }
    }
}

to_dirs!(N); to_dirs!(NE); to_dirs!(E); to_dirs!(SE);
to_dirs!(S); to_dirs!(SW); to_dirs!(W); to_dirs!(NW);

impl ToDirections for N_ {
    fn to_directions(&self) -> Vec<Direction> {
        vec![Direction::N, Direction::NE, Direction::NW]
    }
}

impl ToDirections for S_ {
    fn to_directions(&self) -> Vec<Direction> {
        vec![Direction::S, Direction::SE, Direction::SW]
    }
}

impl ToDirections for _E {
    fn to_directions(&self) -> Vec<Direction> {
        vec![Direction::E, Direction::NE, Direction::SE]
    }
}

impl ToDirections for _W {
    fn to_directions(&self) -> Vec<Direction> {
        vec![Direction::W, Direction::NW, Direction::SW]
    }
}
```
