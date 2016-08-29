The `find_path` module holds the basic primitives for
finding paths in a given `Grid`.

Finding a path involves accumulating state (e.g. the series of
points that make up the path).

```rust
use directions::{Direction, DIRECTIONS};
use format::Table;
use grid::{Elem, Grid, Pt, DirVector};
use path::{Closed, Path};

use std::borrow::Cow;

struct FindPaths<'a> {
    format: Cow<'a, Table>,
    grid: &'a Grid,
    steps: Vec<Pt>,
}

#[derive(Debug)]
struct FindClosedPaths<'a> {
    find: FindPaths<'a>
}

#[derive(Debug)]
struct FindUnclosedPaths<'a> {
    find: FindPaths<'a>
}

use std;

impl<'a> std::fmt::Debug for FindPaths<'a> {
    fn fmt(&self, w: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(w, "FindPaths {{ grid, steps: {:?} }}", self.steps)
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
enum FindContextKind {
    Start,
    TurnAny(Direction),
    Trajectory(Direction),
}

#[derive(Debug)]
struct FindContext {
    prev: Option<Pt>,
    curr: Pt,
    kind: FindContextKind,
}

impl FindContext {
    fn curr(&self) -> Pt { self.curr }
}

impl<'a> FindPaths<'a> {
```

Exercise note: When taking only `&Grid`,
the return-type needs to be `FindPaths`, not `Self`.

Some day, It may be worthwhile to make an exercise out of why.
(A strong hint is present in the signature of `fn with_grid_format`)

```rust
    #[cfg(test)]
    fn new(grid: &Grid) -> FindPaths {
        FindPaths {
            format: Default::default(),
            grid: grid,
            steps: vec![],
        }
    }

    fn with_grid_format(grid: &'a Grid, format: &'a Table) -> Self {
        FindPaths {
            format: Cow::Borrowed(format),
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
}

impl<'a> FindUnclosedPaths<'a> {
    fn find_unclosed_path(mut self, curr: Pt) -> Result<Path, Self> {
        let elem = self.find.grid[curr];
        debug!("find_unclosed_path self: {:?} curr: {:?} pt: {:?}", self, curr, elem);
        // don't waste time on a search that starts on a blank cell
        if elem.is_blank() {
            return Err(self);
        }
        // don't waste time on a search that starts on a cell with no non-blank neighbors.
        {
            let mut non_blank_nbors = 0;
            for &dir in DIRECTIONS.iter() {
                let next = DirVector(curr, dir).steps(1);
                if !self.find.grid.holds(next.0) { continue; }
                if !self.find.grid[next.0].is_blank() { non_blank_nbors += 1; }
            }
            if non_blank_nbors == 0 {
                debug!("find_unclosed_path: early exit on {:?} at {:?} with {} neighbors.", elem, curr, non_blank_nbors);
                return Err(self);
            }
        }

        // start the search proper
        self.find.steps.push(curr);
        for (j, &dir) in DIRECTIONS.iter().enumerate() {
            debug!("find_unclosed_path {} dir: {:?}",
                   j, dir);
            let next = DirVector(curr, dir).steps(1);
            debug!("find_unclosed_path {} dir: {:?} next: {:?}",
                  j, dir, next);
            if !self.find.grid.holds(next.0) {
                continue;
            }
            match self.find_unclosed_path_from(next, FindContext {
                prev: Some(curr), curr: next.0, kind: FindContextKind::Start })
            {
                ret @ Ok(_) => {
                    return ret;
                }
                Err(s) => { self = s; }
            }
        }
        debug!("find_unclosed_path self: {:?} exhausted directions; giving up.", self);
        return Err(self);
    }

    fn find_unclosed_path_from(mut self, dv: DirVector, fc: FindContext) -> Result<Path, Self> {
        use self::FindContextKind::*;
        use path::Closed::*;
        debug!("find_unclosed_path_from self: {:?} dv: {:?} {:?}", self, dv, fc);
        assert!(self.find.grid.holds(dv.0));
        assert!(!self.find.steps.contains(&dv.0));
        assert_eq!(dv.0, fc.curr());
        let elem: Elem = self.find.grid[dv.0];
        debug!("find_unclosed_path_from elem: {:?}", elem);
        let _c: char = match elem {
            Elem::Pad | Elem::Clear => { // blank: Give up.
                debug!("find_unclosed_path self: {:?} blank; giving up.", self);
                return Err(self);
            }
            Elem::C(c) | Elem::Used(c) => c,
        };

        // attempt to continue along our current trajectory.
        let dir = dv.dir();
        let mut dirs_to_try: Vec<_> = DIRECTIONS.iter().map(|&d|d).filter(|&d| d != dir && d != dir.reverse()).collect();
        dirs_to_try.push(dir);
        // TODO: maybe experiment with a non-clock iteration order,
        // such as CW, 2*CCW, 3*CW, 4*CCW, ...

        // add `curr` to path.
        self.find.steps.push(dv.0);

        while let Some(dir) = dirs_to_try.pop() {
            debug!("find_unclosed_path trying dir: {:?}", dir);
            let next = dv.towards(dir).step();

            if !self.find.grid.holds(next.0) { continue; } // off grid
            if self.find.steps.contains(&next.0) { continue; } // overlap
            if !self.find.matches(fc.prev, fc.curr, next.0) { continue; } // no format rule


            match self.find_unclosed_path_from(next, FindContext { prev: Some(dv.0),
                                                                   curr: next.0,
                                                                   kind: TurnAny(dir) }) {
                p @ Ok(_) => return p,
                Err(s) => {
                    self = s;
                    continue;
                }
            }
        }

        // If we get here, then none of the available directions worked, so attempt finish.
        assert_eq!(self.find.steps.last(), Some(&dv.0));
        if self.find.matches_end(fc.prev, fc.curr) {
            debug!("find_unclosed_path self: {:?} exhausted turns; finished.", self);
            Ok(self.find.to_path(Open))
        } else {
            debug!("find_unclosed_path self: {:?} cannot end here; aborting.", self);
            // undo addition of `curr` to path
            assert_eq!(self.find.steps.last(), Some(&dv.0));
            self.find.steps.pop();
            Err(self)
        }
    }
}

impl<'a> FindClosedPaths<'a> {
    fn is_corner(&self, curr: Pt) -> Option<Vec<((char, Direction), (Direction, char))>> {
        let grid = &self.find.grid;
        let c = match grid[curr] {
            Elem::C(c) | Elem::Used(c) => c,
            Elem::Pad | Elem::Clear => {
                debug!("elem {:?} found, is_corner returns None", grid[curr]);
                return None;
            }
        };
        let mut in_out = Vec::new();
        for entry in &self.find.format.entries {
            if !entry.loop_start { continue; }
            if !entry.matches_curr(c) { continue; }

            // This is a *potential* corner.
            //
            // check that there exist distinct neighbors that work
            // with the incoming and outgoing parts of the entry.
            let (in_match, in_dir) = entry.corner_incoming();
            let (out_dir, out_match) = entry.corner_outgoing();
            for in_dir in in_dir {
                for &out_dir in &out_dir {
                    if in_dir == out_dir { continue; }
                    let i = curr.neighbor(in_dir);
                    let o = curr.neighbor(out_dir);
                    if !grid.holds(i) || !grid.holds(o) { continue; }
                    if let (Some(i), Some(o)) = (grid[i].opt_char(), grid[o].opt_char()) {
                        if in_match.matches(i) && out_match.matches(o) {
                            in_out.push(((i, in_dir), (out_dir, o)));
                        }
                    }
                }
            }
        }

        return if in_out.is_empty() {
            debug!("no in_out found, is_corner returns None");
            None
        } else {
            Some(in_out)
        };
    }

    fn find_closed_path(&mut self, curr: Pt) -> Option<Path> {
        let elem = self.find.grid[curr];
        debug!("find_closed_path self: {:?} curr: {:?} pt: {:?}", self, curr, elem);
        // Don't waste time on a search that starts on a non-corner.
        // (all closed paths must have at least three corner elements,
        //  since we need three points to define a positive 2D area;
        //  so we can be assured that at least one corner exists
        //  somewhere.)
        let corner_dirs = match self.is_corner(curr) {
            None => {
                debug!("find_closed_path: early exit on non-corner: {:?} at {:?}", elem, curr);
                return None;
            }
            Some(v) => v,
        };
        // Also, don't waste time on a search that starts on a cell with < 2 non-blank neighbors.
        {
            let mut non_blank_nbors = 0;
            for &dir in DIRECTIONS.iter() {
                let next = DirVector(curr, dir).steps(1);
                if !self.find.grid.holds(next.0) { continue; }
                if !self.find.grid[next.0].is_blank() { non_blank_nbors += 1; }
            }
            if non_blank_nbors < 2 {
                unreachable!(); // (this became unreachable with the self.is_corner change above)
            }
        }

        // start the search proper
        self.find.steps.push(curr);
        for (j, &dir) in DIRECTIONS.iter().enumerate() {
            debug!("find_closed_path {} dir: {:?}",
                  j, dir);
            let next = DirVector(curr, dir).steps(1);
            debug!("find_closed_path {} dir: {:?} next: {:?}",
                  j, dir, next);
            if !self.find.grid.holds(next.0) {
                continue;
            } else if next.0 == self.find.start() {
                return Some(self.find.to_path(Closed::Closed));
            }
            if let ret @ Some(_) = self.find_closed_path_from(next, FindContext {
                prev: Some(curr),
                curr: next.0,
                kind: FindContextKind::Start
            }) {
                return ret;
            }
        }
        debug!("find_closed_path self: {:?} exhausted directions; giving up.", self);
        return None;
    }

    fn find_closed_path_from(&mut self, dv: DirVector, fc: FindContext) -> Option<Path> {
        use self::Continue::*;
        use self::FindContextKind::*;
        use directions::Turn;
        debug!("find_closed_path_from self: {:?} dv: {:?} {:?}", self, dv, fc);
        assert!(self.find.grid.holds(dv.0));
        assert!(!self.find.steps.contains(&dv.0));
        assert_eq!(dv.0, fc.curr());
        let elem: Elem = self.find.grid[dv.0];
        debug!("find_closed_path_from elem: {:?}", elem);
        return match elem {
            Elem::C(c) | Elem::Used(c) => {
                let cont = self::Continue::cat(c);
                debug!("find_closed_path_from elem: {:?} cont: {:?}", elem, cont);
                if cont == AnyDir {
                    self.find.steps.push(dv.0);
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
                        match self.try_next(next, FindContext {
                            prev: Some(dv.0),
                            curr: next.0,
                            kind: TurnAny(dir) })
                        {
                            p @ Some(_) => return p,
                            None => {
                                dir = dir.veer(Turn::CCW);
                            }
                        }
                    }

                    // If we get here, then none of the available directions
                    // worked, so give up.
                    assert_eq!(self.find.steps.last(), Some(&dv.0));
                    self.find.steps.pop();
                    debug!("find_closed_path self: {:?} exhausted turns; giving up.", self);
                    None

                } else if cont.matches(dv.1) && cont != AnyDir {
                    // If we cannot take any direction from the given
                    // point, but this point in the grid is consistent
                    // with our current trajectory, then attempt to
                    // continue in the same trajectory.
                    self.find.steps.push(dv.0);
                    let next = dv.steps(1);
                    match self.try_next(next, FindContext {
                        prev: Some(dv.0),
                        curr: next.0,
                        kind: Trajectory(next.1)})
                    {
                        p @ Some(_) => p,
                        None => {
                            assert_eq!(self.find.steps.last(), Some(&dv.0));
                            self.find.steps.pop();
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
}

impl<'a> FindClosedPaths<'a> {
    #[cfg(test)]
    fn new(grid: &'a Grid) -> Self {
        FindClosedPaths { find: FindPaths::new(grid) }
    }
    fn try_next(&mut self, next: DirVector, fc: FindContext) -> Option<Path> {
        debug!("try_next Closed self: {:?} next: {:?} {:?}", self, next, fc);

        if !self.find.grid.holds(next.0) { // off grid
            return None
        } else if self.find.start() == next.0 { // closes path; success!
            return Some(self.find.to_path(Closed::Closed))
        } else if self.find.steps.contains(&next.0) { // non-start overlap
            return None
        }

        self.find_closed_path_from(next, fc)
    }
}

impl<'a> FindUnclosedPaths<'a> {
    fn try_next(self, next: DirVector, fc: FindContext) -> Result<Path, Self> {
        debug!("try_next Unclosed self: {:?} next: {:?} {:?}", self, next, fc);

        if !self.find.grid.holds(next.0) { // off grid
            return Err(self);
        } else if self.find.steps.contains(&next.0) { // non-start overlap
            return Err(self);
        }

        self.find_unclosed_path_from(next, fc)
    }
}

/// Variants of `Continue` categorize how a given character can
/// extend a path.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Continue {
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
    fn cat(c: char) -> Continue {
        use self::Continue::*;
        match c {
            '\\' => NegSlope,
            '/' => PosSlope,
            '|' | ':' | '^' | 'v' => Vertical,
            '-' | '=' | '>' | '<' => Horizontal,
            '*' => Continue,
            '.' | '\'' | '+' => AnyDir,
            _ => End,
        }
    }

    /// Answers whether this continuation category can extend in the
    /// given direction `dir`.
    fn matches(&self, dir: Direction) -> bool {
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

pub fn find_closed_path(grid: &Grid, format: &Table, pt: Pt) -> Option<Path> {
    let mut pf = FindClosedPaths { find: FindPaths::with_grid_format(grid, format) };
    pf.find_closed_path(pt)
}

pub fn find_unclosed_path_from(grid: &Grid, format: &Table, dir: DirVector) -> Option<Path> {
    let pf = FindUnclosedPaths { find: FindPaths::with_grid_format(grid, format) };
    pf.find_unclosed_path_from(dir, FindContext {
        prev: None,
        curr: dir.0,
        kind: FindContextKind::Start
    }).ok()
}

pub fn find_unclosed_path(grid: &Grid, format: &Table, pt: Pt) -> Option<Path> {
    let pf = FindUnclosedPaths { find: FindPaths::with_grid_format(grid, format) };
    pf.find_unclosed_path(pt).ok()
}

impl<'a> FindPaths<'a> {
    fn matches(&self, prev: Option<Pt>, curr: Pt, next: Pt) -> bool {
        let c = if let Some(c) = self.grid[curr].opt_char() { c } else { return false; };
        let n = self.grid[next];
        let prev_arc = prev.and_then(|prev| {
            self.grid[prev].opt_char().map(|p| (p, prev.towards(curr)))
        });
        let next_arc = self.grid[next].opt_char().map(|n| (curr.towards(next), n));
        for entry in &self.format.entries {
            if entry.matches(prev_arc, c, next_arc) {
                return true;
            }
        }
        return false;
    }

    fn matches_end(&self, prev: Option<Pt>, curr: Pt) -> bool {
        let c = if let Some(c) = self.grid[curr].opt_char() { c } else { return false; };
        let prev_arc = prev.and_then(|prev| {
            self.grid[prev].opt_char().map(|p| (p, prev.towards(curr)))
        });
        for entry in &self.format.entries {
            if entry.matches_end(prev_arc, c) {
                return true;
            }
        }
        return false;
    }
}

#[cfg(test)]
mod tests;
```
