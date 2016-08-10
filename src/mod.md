The `find_path` module holds the basic primitives for
finding paths in a given `Grid`.

Finding a path involves accumulating state (e.g. the series of
points that make up the path).

```rust
use grid::{Direction, DIRECTIONS, DirVector, Elem, Grid, Pt};
use path::{Closed, Path};

#[derive(Clone, Hash)]
pub struct FindPaths<'a> {
    grid: &'a Grid,
    steps: Vec<Pt>,
}

use std;

impl<'a> std::fmt::Debug for FindPaths<'a> {
    fn fmt(&self, w: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(w, "PathFinder {{ grid, steps: {:?} }}", self.steps)
    }
}
```

There are a couple different basic strategies for ensure that we
only seek *new* trails when extending a path. We can either:

 * Mutate the grid as we go, e.g. putting blank spaces in every
   non-crossing point that we make part of our current path, so that
   future attempts to traverse that point will cause the path to
   terminate

 * Leave the grid unmutated, and instead query the accumulated
   set of points on each traversal, to see if the point in question
   is already part of our current path.

Note: A crossing point is a place where a path is allow to intersect
another path. A typical ascii-art example is the `+` character, as in:

```
 |
-+-
 |
```

but we might want to select a different one, such as `#`, and reserve
`+` solely for corners in paths. See discussion at [a2s issue 15][].

[a2s issue 15]: https://bitbucket.org/dhobsd/asciitosvg/issues/15/boxes-with-corners-cause-a-memory-leak

I guess the first option could be (arguably) a special case of the
second, if we were to mutate a *local copy* of the grid.

In any case, the main drawback to that variant is that when we
back-track, we will need to restore the grid to its previous
state. The backtracking of the second approach is just a matter of
popping the vector of points that we are building up.

```rust
#[derive(Debug)]
enum FindContext {
    Start,
    TurnAny(Direction),
    Trajectory(Direction),
}

impl<'a> FindPaths<'a> {
```

Exercise note: This return-type needs to be `FindPaths`, not `Self`.
Some day, It may be worthwhile to make an exercise out of why.

```rust
    fn new(grid: &Grid) -> FindPaths {
        FindPaths {
            grid: grid,
            steps: vec![],
        }
    }

    fn start(&self) -> Pt { self.steps[0] }

    fn to_path(&self, closed: Closed) -> Path {
        Path {
            // it would be nice to figure out an API that converted
            // self *into* a Path to avoid this clone. But I expect
            // the path lengths to all be so short that this won't
            // matter.
            steps: self.steps.iter().map(|&pt| (pt, self.grid[pt].to_char())).collect(),
            closed: closed,
            id: None,
            attrs: None,
        }
    }

    fn find_unclosed_path_from(&mut self, dv: DirVector, fc: FindContext) -> Option<Path> {
        use self::Continue::*;
        use path::Closed::*;
        use grid::Turn;
        debug!("find_unclosed_path_from self: {:?} dv: {:?} {:?}", self, dv, fc);
        assert!(self.grid.holds(dv.0));
        assert!(!self.steps.contains(&dv.0));
        let elem: Elem = self.grid[dv.0];
        debug!("find_unclosed_path_from elem: {:?}", elem);
        return match elem {
            Elem::C(c) => {
                let cont = self::Continue::cat(c);
                debug!("find_unclosed_path_from elem: {:?} cont: {:?}", elem, cont);
                if cont == AnyDir {
                    self.steps.push(dv.0);
                    // if we can turn in any direction, attempt to
                    // continue along our current trajectory.
                    let mut dir = dv.dir();
                    let mut tried_self = false;
                    while !tried_self || dir != dv.dir() {
                        debug!("find_unclosed_path trying dir: {:?}", dir);
                        if dir == dv.dir() { tried_self = true; }
                        if dir == dv.dir().reverse() { continue; }
                        let next = dv.towards(dir).step();
                        match self.try_next(next, Open, FindContext::TurnAny(dir)) {
                            p @ Some(_) => return p,
                            None => {
                                // TODO: maybe experiment with a non-clock iteration order,
                                // such as CW, 2*CCW, 3*CW, 4*CCW, ...
                                dir = dir.veer(Turn::CW);
                            }
                        }
                    }

                    // If we get here, then none of the available directions worked, so finish.
                    assert_eq!(self.steps.last(), Some(&dv.0));
                    debug!("find_unclosed_path self: {:?} exhausted turns; finished.", self);
                    Some(self.to_path(Open))

                } else if cont.matches(dv.1) && cont != AnyDir {
                    self.steps.push(dv.0);
                    let next = dv.steps(1);
                    match self.try_next(next, Open, FindContext::Trajectory(next.1)) {
                        p @ Some(_) => p,
                        None => {
                            assert_eq!(self.steps.last(), Some(&dv.0));
                            debug!("find_unclosed_path self: {:?} following trajectory failed; finished.", self);
                            Some(self.to_path(Open))
                        }
                    }
                } else {
                    debug!("find_unclosed_path self: {:?} unmatched trajectory; giving up.", self);
                    None
                }
            }
            Elem::Pad | Elem::Clear => { // blank: Give up.
                debug!("find_unclosed_path self: {:?} blank; giving up.", self);
                None
            }
        };
    }

    fn find_closed_path(&mut self, curr: Pt) -> Option<Path> {
        debug!("find_closed_path self: {:?} curr: {:?} pt: {:?}", self, curr, self.grid[curr]);
        self.steps.push(curr);
        for (j, &dir) in DIRECTIONS.iter().enumerate() {
            debug!("find_closed_path {} dir: {:?}",
                  j, dir);
            let next = DirVector(curr, dir).steps(1);
            debug!("find_closed_path {} dir: {:?} next: {:?}",
                  j, dir, next);
            if !self.grid.holds(next.0) {
                continue;
            } else if next.0 == self.start() {
                return Some(self.to_path(Closed::Closed));
            }
            if let ret @ Some(_) = self.find_closed_path_from(next, FindContext::Start) {
                return ret;
            }
        }
        debug!("find_closed_path self: {:?} exhausted directions; giving up.", self);
        return None;
    }

    fn find_closed_path_from(&mut self, dv: DirVector, fc: FindContext) -> Option<Path> {
        use self::Continue::*;
        use path::Closed::*;
        use grid::Turn;
        debug!("find_closed_path_from self: {:?} dv: {:?} {:?}", self, dv, fc);
        assert!(self.grid.holds(dv.0));
        assert!(!self.steps.contains(&dv.0));
        let elem: Elem = self.grid[dv.0];
        debug!("find_closed_path_from elem: {:?}", elem);
        return match elem {
            Elem::C(c) => {
                let cont = self::Continue::cat(c);
                debug!("find_closed_path_from elem: {:?} cont: {:?}", elem, cont);
                if cont == AnyDir {
                    self.steps.push(dv.0);
                    // if we can turn in any direction, attempt to
                    // make the closed polygon via the sharpest
                    // clockwise turn possible.
                    let mut dir = dv.dir().sharp_turn(Turn::CW);
                    // (If that fails, we will try the next sharpest
                    // by veering counter-clockwise, up until (but not
                    // including) we end up going in the entirely
                    // reverse direction from where we started.)
                    while dir != dv.dir().reverse() {
                        let next = dv.towards(dir).step();
                        match self.try_next(next, Closed, FindContext::TurnAny(dir)) {
                            p @ Some(_) => return p,
                            None => {
                                dir = dir.veer(Turn::CCW);
                            }
                        }
                    }

                    // If we get here, then none of the available directions
                    // worked, so give up.
                    assert_eq!(self.steps.last(), Some(&dv.0));
                    self.steps.pop();
                    debug!("find_closed_path self: {:?} exhausted turns; giving up.", self);
                    None

                } else if cont.matches(dv.1) && cont != AnyDir {
                    // If we cannot take any direction from the given
                    // point, but this point in the grid is consistent
                    // with our current trajectory, then attempt to
                    // continue in the same trajectory.
                    self.steps.push(dv.0);
                    let next = dv.steps(1);
                    match self.try_next(next, Closed, FindContext::Trajectory(next.1)) {
                        p @ Some(_) => p,
                        None => {
                            assert_eq!(self.steps.last(), Some(&dv.0));
                            self.steps.pop();
                            debug!("find_closed_path self: {:?} following trajectory failed; giving up.", self);
                            None
                        }
                    }
                } else {
                    // If this point in the grid did not match our
                    // trajectory, then this cannot be a continuation
                    // of our current path. Give up.
                    debug!("find_closed_path self: {:?} unmatched trajectory; giving up.", self);
                    None
                }
            }
            Elem::Pad | Elem::Clear => { // blank: Give up.
                debug!("find_closed_path self: {:?} blank; giving up.", self);
                None
            }
        };
    }

    fn try_next(&mut self, next: DirVector, closed: Closed, fc: FindContext) -> Option<Path> {
        debug!("try_next self: {:?} next: {:?} closed: {:?} {:?}", self, next, closed, fc);

        if !self.grid.holds(next.0) { // off grid
            return None
        } else if self.start() == next.0 && closed == Closed::Closed { // closes path; success!
            return Some(self.to_path(Closed::Closed))
        } else if self.steps.contains(&next.0) { // non-start overlap
            return None
        }

        match closed {
            Closed::Closed => self.find_closed_path_from(next, fc),
            Closed::Open => self.find_unclosed_path_from(next, fc),
        }
    }
}

/// Variants of `Continue` categorize how a given character can
/// extend a path.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Continue {
    /// A char like `|` or `:` requires continuing vertically.
    Vertical,

    /// A char like `-` or `=` requires continuing vertically.
    Horizontal,

    /// A char like `/` requires continuing northeast or southwest.
    PosSlope,

    /// A char like `\` requires continuing northwest or southeast.
    NegSlope,

    /// Some characters (e.g. `*`?) work in any context but just mean
    /// to continue (linearly) the same way that we were going.
    Continue,

    /// Some characters like `+` serve as junction points where we can
    /// turn in arbitrary directions.
    ///
    /// (For now `.` and `'` also allow arbitrary directions, but in
    /// the future I plan to add `AnyButNorth` and `AnyButSouth`
    /// variants that will restrict how we can continue on from `.`
    /// and `'`.)
    AnyDir,
    // AnyButSouth,
    // AnyButNorth,

    /// *Most* characters (e.g. whitespace, text) mean that we cannot
    /// continue in any direction, and that this is the end of the
    /// path (or a dead-end, in the case of a closed path).
    End,
}

impl Continue {
    /// Categorize how the given char can extend a path.
    pub fn cat(c: char) -> Continue {
        use self::Continue::*;
        match c {
            '\\' => NegSlope,
            '/' => PosSlope,
            '|' | ':' => Vertical,
            '-' | '=' => Horizontal,
            '*' => Continue,
            '.' | '\'' | '+' => AnyDir,
            _ => End,
        }
    }

    /// Answers whether this continuation category can extend in the
    /// given direction `dir`.
    pub fn matches(&self, dir: Direction) -> bool {
        use self::Continue::*;
        use grid::Direction::*;
        match *self {
            Vertical => match dir { N | S => true, _ => false },
            Horizontal => match dir { W | E => true, _ => false },
            PosSlope => match dir { NE | SW => true, _ => false },
            NegSlope => match dir { NW | SE => true, _ => false },
            Continue => true,
            AnyDir => true,
            End => false,
        }
    }
}

pub fn find_closed_path(grid: &Grid, pt: Pt) -> Option<Path> {
    let mut pf = FindPaths::new(grid);
    pf.find_closed_path(pt)
}

pub fn find_unclosed_path_from(grid: &Grid, dir: DirVector) -> Option<Path> {
    let mut pf = FindPaths::new(grid);
    pf.find_unclosed_path_from(dir, FindContext::Start)
}

#[cfg(not_yet)]
pub fn find_unclosed_path(grid: &Grid, pt: Pt) -> Option<Path> {
    let mut pf = FindPaths::new(grid);
    pf.find_unclosed_path(pt)
}

#[cfg(test)]
mod tests {
    use grid::{Grid, Pt, Direction, DirVector};
    use test_data::{BASIC, BASIC_WO_BOX, BASIC_UL_PLUS, BASIC_UR_PLUS, BASIC_ALL_PLUS};
    use test_data::{ISSUE_15_DESC};
    use path::{Path, Closed};
    use super::FindPaths;

    impl Path {
        fn closed(steps: Vec<(Pt, char)>) -> Path {
            Path { steps: steps, closed: Closed::Closed, id: None, attrs: None, }
        }

        fn open(steps: Vec<(Pt, char)>) -> Path {
            Path { steps: steps, closed: Closed::Open, id: None, attrs: None }
        }
    }

    #[test]
    fn trivial_path_east() {
        let grid = "--- ".parse::<Grid>().unwrap();
        let opt_p = super::find_unclosed_path_from(&grid, DirVector(Pt(1,1), Direction::E));
        assert_eq!(opt_p.unwrap(),
                   Path::open(vec![(Pt(1,1), '-'), (Pt(2,1), '-'), (Pt(3,1), '-')]));
    }

    #[test]
    fn trivial_path_west() {
        let grid = "--- ".parse::<Grid>().unwrap();
        let opt_p = super::find_unclosed_path_from(&grid, DirVector(Pt(3,1), Direction::W));
        assert_eq!(opt_p.unwrap(),
                   Path::open(vec![(Pt(3,1), '-'), (Pt(2,1), '-'), (Pt(1,1), '-')]));
    }

    #[test]
    fn hopping_path_east() {
        ::env_logger::init();
        let grid = "-+- ".parse::<Grid>().unwrap();
        let opt_p = super::find_unclosed_path_from(&grid, DirVector(Pt(1,1), Direction::E));
        assert_eq!(opt_p.unwrap(),
                   Path::open(vec![(Pt(1,1), '-'), (Pt(2,1), '+'), (Pt(3,1), '-')]));
    }

    #[test]
    fn basic_single_box_upper_left() {
        let grid = BASIC.parse::<Grid>().unwrap();
        let opt_p = {
            let mut pf = FindPaths::new(&grid);
            pf.find_closed_path(Pt(1,1))
        };
        assert_eq!(opt_p.clone().unwrap(),
                   Path::closed(vec![(Pt(1,1), '.'), (Pt(2,1), '-'), (Pt(3,1), '-'), (Pt(4,1), '-'), (Pt(5,1), '-'), (Pt(6,1), '.'),
                                     (Pt(6,2), '|'),
                                     (Pt(6,3), '\''), (Pt(5,3), '-'), (Pt(4,3), '-'), (Pt(3,3), '-'), (Pt(2,3), '-'), (Pt(1,3), '\''),
                                     (Pt(1,2), '|')]));
        let mut grid = grid;
        grid.remove_path(&opt_p.unwrap());
        assert_eq!(grid.to_string(), BASIC_WO_BOX);
    }

    #[test]
    fn basic_ul_plus_single_box_upper_left() {
        let grid = BASIC_UL_PLUS.parse::<Grid>().unwrap();
        let mut pf = FindPaths::new(&grid);
        let opt_p = pf.find_closed_path(Pt(1,1));
        assert_eq!(opt_p.unwrap(),
                   Path::closed(vec![(Pt(1,1), '+'), (Pt(2,1), '-'), (Pt(3,1), '-'), (Pt(4,1), '-'), (Pt(5,1), '-'), (Pt(6,1), '.'),
                                     (Pt(6,2), '|'),
                                     (Pt(6,3), '\''), (Pt(5,3), '-'), (Pt(4,3), '-'), (Pt(3,3), '-'), (Pt(2,3), '-'), (Pt(1,3), '\''),
                                     (Pt(1,2), '|')]))
    }

    #[test]
    fn basic_ur_plus_single_box_upper_left() {
        // ::env_logger::init();
        let grid = BASIC_UR_PLUS.parse::<Grid>().unwrap();
        let mut pf = FindPaths::new(&grid);
        let opt_p = pf.find_closed_path(Pt(1,1));
        assert_eq!(opt_p.unwrap(),
                   Path::closed(vec![(Pt(1,1), '.'), (Pt(2,1), '-'), (Pt(3,1), '-'), (Pt(4,1), '-'), (Pt(5,1), '-'), (Pt(6,1), '+'),
                                     (Pt(6,2), '|'),
                                     (Pt(6,3), '\''), (Pt(5,3), '-'), (Pt(4,3), '-'), (Pt(3,3), '-'), (Pt(2,3), '-'), (Pt(1,3), '\''),
                                     (Pt(1,2), '|')]))
    }

    #[test]
    fn basic_all_plus_single_box_upper_left() {
        let grid = BASIC_ALL_PLUS.parse::<Grid>().unwrap();
        let mut pf = FindPaths::new(&grid);
        let opt_p = pf.find_closed_path(Pt(1,1));
        assert_eq!(opt_p.unwrap(),
                   Path::closed(vec![(Pt(1,1), '+'), (Pt(2,1), '-'), (Pt(3,1), '-'), (Pt(4,1), '-'), (Pt(5,1), '-'), (Pt(6,1), '+'),
                                     (Pt(6,2), '|'),
                                     (Pt(6,3), '+'), (Pt(5,3), '-'), (Pt(4,3), '-'), (Pt(3,3), '-'), (Pt(2,3), '-'), (Pt(1,3), '+'),
                                     (Pt(1,2), '|')]))
    }

    #[test]
    fn issue_15_box_big_upper_left() {
        let grid = ISSUE_15_DESC.parse::<Grid>().unwrap();
        let mut pf = FindPaths::new(&grid);
        let opt_p = pf.find_closed_path(Pt(1, 2));
        assert_eq!(opt_p.unwrap(),
                   Path::closed(vec![(Pt(1,2), '.'), (Pt(2,2), '-'), (Pt(3,2), '-'), (Pt(4,2), '-'),
                                     (Pt(5,2), '-'), (Pt(6,2), '-'), (Pt(7,2), '-'), (Pt(8,2), '-'),
                                     (Pt(9,2), '-'), (Pt(10,2), '-'), (Pt(11,2), '-'), (Pt(12,2), '-'),
                                     (Pt(13,2), '-'), (Pt(14,2), '-'), (Pt(15,2), '.'),
                                     (Pt(15,3), '|'),
                                     (Pt(15,4), '+'),
                                     (Pt(15,5), '|'),
                                     (Pt(15,6), '\''), (Pt(14,6), '-'), (Pt(13,6), '-'), (Pt(12,6), '-'),
                                     (Pt(11,6), '-'), (Pt(10,6), '-'), (Pt(9,6), '-'), (Pt(8,6), '-'),
                                     (Pt(7,6), '+'), (Pt(6,6), '-'), (Pt(5,6), '-'), (Pt(4,6), '-'),
                                     (Pt(3,6), '-'), (Pt(2,6), '-'), (Pt(1,6), '\''),
                                     (Pt(1,5), '|'),
                                     (Pt(1,4), '|'),
                                     (Pt(1,3), '|')]));
    }

    #[test]
    fn issue_15_box_lil_lower_left() {
        let grid = ISSUE_15_DESC.parse::<Grid>().unwrap();
        let mut pf = FindPaths::new(&grid);
        let opt_p = pf.find_closed_path(Pt(5, 11));
        assert_eq!(opt_p.unwrap(),
                   Path::closed(vec![(Pt(5,11), '.'), (Pt(6,11), '-'), (Pt(7,11), '+'), (Pt(8,11), '-'), (Pt(9,11), '.'),
                                     (Pt(9,12), '|'),
                                     (Pt(9,13), '\''), (Pt(8,13), '-'), (Pt(7,13), '-'), (Pt(6,13), '-'), (Pt(5,13), '\''),
                                     (Pt(5,12), '|')]));
    }

    #[test]
    fn issue_15_box_big_upper_middle() {
        // ::env_logger::init();
        let grid = ISSUE_15_DESC.parse::<Grid>().unwrap();
        let mut pf = FindPaths::new(&grid);
        let opt_p = pf.find_closed_path(Pt(25, 4));
        assert_eq!(opt_p.unwrap(),
                   Path::closed(vec![(Pt(25,4), '.'), (Pt(26,4), '-'), (Pt(27,4), '-'), (Pt(28,4), '-'), (Pt(29,4), '-'),
                                     (Pt(30,4), '-'), (Pt(31,4), '-'), (Pt(32,4), '-'), (Pt(33,4), '-'), (Pt(34,4), '-'),
                                     (Pt(35,4), '-'), (Pt(36,4), '-'), (Pt(37,4), '-'), (Pt(38,4), '-'), (Pt(39,4), '.'),
                                     (Pt(39,5), '|'),
                                     (Pt(39,6), '+'),
                                     (Pt(39,7), '|'),
                                     (Pt(39,8), '|'),
                                     (Pt(39,9), '\''), (Pt(38,9), '-'), (Pt(37,9), '-'), (Pt(36,9), '-'), (Pt(35,9), '-'),
                                     (Pt(34,9), '-'), (Pt(33,9), '-'), (Pt(32,9), '-'), (Pt(31,9), '-'), (Pt(30,9), '-'),
                                     (Pt(29,9), '-'), (Pt(28,9), '-'), (Pt(27,9), '-'), (Pt(26,9), '-'), (Pt(25,9), '\''),
                                     (Pt(25,8), '|'),
                                     (Pt(25,7), '+'),
                                     (Pt(25,6), '|'),
                                     (Pt(25,5), '|')]));
    }

    #[test]
    fn issue_15_box_big_lower_right() {
        // ::env_logger::init().ok(); // discard error since double-init is one.
        let grid = ISSUE_15_DESC.parse::<Grid>().unwrap();
        let mut pf = FindPaths::new(&grid);
        let opt_p = pf.find_closed_path(Pt(35, 13));
        assert_eq!(opt_p.unwrap(),
                   Path::closed(vec![(Pt(35,13), '+'), (Pt(36,13), '-'), (Pt(37,13), '-'), (Pt(38,13), '-'), (Pt(39,13), '-'),
                                     (Pt(40,13), '-'), (Pt(41,13), '-'), (Pt(42,13), '-'), (Pt(43,13), '-'), (Pt(44,13), '-'),
                                     (Pt(45,13), '-'), (Pt(46,13), '-'), (Pt(47,13), '-'), (Pt(48,13), '-'), (Pt(49,13), '+'),
                                     (Pt(50,13), '-'), (Pt(51,13), '-'), (Pt(52,13), '-'), (Pt(53,13), '-'), (Pt(54,13), '-'),
                                     (Pt(55,13), '-'), (Pt(56,13), '+'),
                                     (Pt(56,14), '|'),
                                     (Pt(56,15), '|'),
                                     (Pt(56,16), '|'),
                                     (Pt(56,17), '|'),
                                     (Pt(56,18), '+'), (Pt(55,18), '-'), (Pt(54,18), '-'), (Pt(53,18), '-'), (Pt(52,18), '-'),
                                     (Pt(51,18), '-'), (Pt(50,18), '-'), (Pt(49,18), '-'), (Pt(48,18), '-'), (Pt(47,18), '-'),
                                     (Pt(46,18), '-'), (Pt(45,18), '-'), (Pt(44,18), '-'), (Pt(43,18), '-'), (Pt(42,18), '-'),
                                     (Pt(41,18), '-'), (Pt(40,18), '-'), (Pt(39,18), '-'), (Pt(38,18), '-'), (Pt(37,18), '-'),
                                     (Pt(36,18), '-'), (Pt(35,18), '+'),
                                     (Pt(35,17), '|'),
                                     (Pt(35,16), '|'),
                                     (Pt(35,15), '|'),
                                     (Pt(35,14), '|')]));
    }

    #[test]
    fn trivial_box_removal() {
        let mut grid = ".--. top\n\
                        |  | mid\n\
                        '--' bot\n".parse::<Grid>().unwrap();
        let path = {
            let mut pf = FindPaths::new(&grid);
            pf.find_closed_path(Pt(1,1)).unwrap()
        };
        grid.remove_path(&path);
        assert_eq!(grid.to_string(),
                   "\u{7f}\u{7f}\u{7f}\u{7f} top\n\
                    \u{7f}  \u{7f} mid\n\
                    \u{7f}\u{7f}\u{7f}\u{7f} bot\n");
    }

    #[test]
    fn box_removal_but_ul_corner() {
        let mut grid = "+--. top\n\
                        |  | mid\n\
                        '--' bot\n".parse::<Grid>().unwrap();
        let path = {
            let mut pf = FindPaths::new(&grid);
            pf.find_closed_path(Pt(1,1)).unwrap()
        };
        grid.remove_path(&path);
        assert_eq!(grid.to_string(),
                   "+\u{7f}\u{7f}\u{7f} top\n\
                    \u{7f}  \u{7f} mid\n\
                    \u{7f}\u{7f}\u{7f}\u{7f} bot\n");
    }

    #[test]
    fn box_removal_but_ur_corner() {
        let mut grid = ".--+ top\n\
                        |  | mid\n\
                        '--' bot\n".parse::<Grid>().unwrap();
        let path = {
            let mut pf = FindPaths::new(&grid);
            pf.find_closed_path(Pt(1,1)).unwrap()
        };
        grid.remove_path(&path);
        assert_eq!(grid.to_string(),
                   "\u{7f}\u{7f}\u{7f}+ top\n\
                    \u{7f}  \u{7f} mid\n\
                    \u{7f}\u{7f}\u{7f}\u{7f} bot\n");
    }

    #[test]
    fn box_removal_but_bl_corner() {
        let mut grid = ".--. top\n\
                        |  | mid\n\
                        +--' bot\n".parse::<Grid>().unwrap();
        let path = {
            let mut pf = FindPaths::new(&grid);
            pf.find_closed_path(Pt(1,1)).unwrap()
        };
        grid.remove_path(&path);
        assert_eq!(grid.to_string(),
                   "\u{7f}\u{7f}\u{7f}\u{7f} top\n\
                    \u{7f}  \u{7f} mid\n\
                    +\u{7f}\u{7f}\u{7f} bot\n");
    }

    #[test]
    fn box_removal_but_br_corner() {
        let mut grid = ".--. top\n\
                        |  | mid\n\
                        '--+ bot\n".parse::<Grid>().unwrap();
        let path = {
            let mut pf = FindPaths::new(&grid);
            pf.find_closed_path(Pt(1,1)).unwrap()
        };
        grid.remove_path(&path);
        assert_eq!(grid.to_string(),
                   "\u{7f}\u{7f}\u{7f}\u{7f} top\n\
                    \u{7f}  \u{7f} mid\n\
                    \u{7f}\u{7f}\u{7f}+ bot\n");
    }
}
```
