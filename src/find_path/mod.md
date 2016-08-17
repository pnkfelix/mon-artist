The `find_path` module holds the basic primitives for
finding paths in a given `Grid`.

Finding a path involves accumulating state (e.g. the series of
points that make up the path).

```rust
use directions::{Direction, DIRECTIONS};
use grid::{Elem, Grid, Pt, DirVector};
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

    fn find_unclosed_path(&mut self, curr: Pt) -> Option<Path> {
        let elem = self.grid[curr];
        debug!("find_unclosed_path self: {:?} curr: {:?} pt: {:?}", self, curr, elem);
        // don't waste time on a search that starts on a blank (or, for now, previously used) cell
        if elem.is_blank() || elem.is_used() {
            return None;
        }
        // don't waste time on a search that starts on a cell with no non-blank neighbors.
        {
            let mut non_blank_nbors = 0;
            for &dir in DIRECTIONS.iter() {
                let next = DirVector(curr, dir).steps(1);
                if !self.grid.holds(next.0) { continue; }
                if !self.grid[next.0].is_blank() { non_blank_nbors += 1; }
            }
            if non_blank_nbors == 0 {
                debug!("find_unclosed_path: early exit on cell with {} neighbors.", non_blank_nbors);
                return None;
            }
        }

        // start the search proper
        self.steps.push(curr);
        for (j, &dir) in DIRECTIONS.iter().enumerate() {
            debug!("find_unclosed_path {} dir: {:?}",
                   j, dir);
            let next = DirVector(curr, dir).steps(1);
            debug!("find_unclosed_path {} dir: {:?} next: {:?}",
                  j, dir, next);
            if !self.grid.holds(next.0) {
                continue;
            }
            if let ret @ Some(_) = self.find_unclosed_path_from(next, FindContext::Start) {
                return ret;
            }
        }
        debug!("find_unclosed_path self: {:?} exhausted directions; giving up.", self);
        return None;
    }

    fn find_unclosed_path_from(&mut self, dv: DirVector, fc: FindContext) -> Option<Path> {
        use self::Continue::*;
        use path::Closed::*;
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
                    let dir = dv.dir();
                    let mut dirs_to_try: Vec<_> = DIRECTIONS.iter().map(|&d|d).filter(|&d| d != dir && d != dir.reverse()).collect();
                    dirs_to_try.push(dir);
                    // TODO: maybe experiment with a non-clock iteration order,
                    // such as CW, 2*CCW, 3*CW, 4*CCW, ...
                    while let Some(dir) = dirs_to_try.pop() {
                        debug!("find_unclosed_path trying dir: {:?}", dir);
                        let next = dv.towards(dir).step();
                        match self.try_next(next, Open, FindContext::TurnAny(dir)) {
                            p @ Some(_) => return p,
                            None => { }
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
            Elem::Used(c) => {
                // hmm, I want to allow some amount of reuse in some cases, but for now it seems to complicate things too
                // much, so don't let unclosed paths continue along used lines.
                debug!("find_unclosed_path self: {:?} previously used c: {}; giving up.", self, c);
                None
            }
            Elem::Pad | Elem::Clear => { // blank: Give up.
                debug!("find_unclosed_path self: {:?} blank; giving up.", self);
                None
            }
        };
    }

    fn find_closed_path(&mut self, curr: Pt) -> Option<Path> {
        let elem = self.grid[curr];
        debug!("find_closed_path self: {:?} curr: {:?} pt: {:?}", self, curr, elem);
        // Don't waste time on a search that starts on a non-corner.
        // (all closed paths must have at least three corner elements,
        //  since we need three points to define a positive 2D area;
        //  so we can be assured that at least one corner exists
        //  somewhere.)
        if !elem.is_corner() {
            debug!("find_closed_path: early exit on non-corner.");
            return None;
        }
        // Also, don't waste time on a search that starts on a cell with < 2 non-blank neighbors.
        {
            let mut non_blank_nbors = 0;
            for &dir in DIRECTIONS.iter() {
                let next = DirVector(curr, dir).steps(1);
                if !self.grid.holds(next.0) { continue; }
                if !self.grid[next.0].is_blank() { non_blank_nbors += 1; }
            }
            if non_blank_nbors < 2 {
                debug!("find_closed_path: early exit on cell with {} neighbors.", non_blank_nbors);
                return None;
            }
        }

        // start the search proper
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
        use directions::Turn;
        debug!("find_closed_path_from self: {:?} dv: {:?} {:?}", self, dv, fc);
        assert!(self.grid.holds(dv.0));
        assert!(!self.steps.contains(&dv.0));
        let elem: Elem = self.grid[dv.0];
        debug!("find_closed_path_from elem: {:?}", elem);
        return match elem {
            Elem::C(c) | Elem::Used(c) => {
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
        use directions::Direction::*;
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

pub fn find_unclosed_path(grid: &Grid, pt: Pt) -> Option<Path> {
    let mut pf = FindPaths::new(grid);
    pf.find_unclosed_path(pt)
}

#[cfg(test)]
mod tests;
```
