The `find_path` module holds the basic primitives for
finding paths in a given `Grid`.

Finding a path involves accumulating state (e.g. the series of
points that make up the path).

```rust
use directions::{Direction, DIRECTIONS};
use format::{self, Table};
use grid::{Elem, Grid, Pt, DirVector};
use path::{Closed, Path};

use std::borrow::Cow;
use std::cell::Cell;

struct FindPaths<'a> {
    announce: Cell<fn (String)>,
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
    prefix_rev: Vec<Pt>,
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
struct FindContext {
    prev: Option<Pt>,
    curr: Pt,
}

#[derive(Debug)]
struct FindLoopContext<'a> {
    prev: Pt,
    curr: Pt,
    corner_dirs: &'a [((char, Direction), (Direction, char))]
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
            announce: Cell::new(silent),
            format: Default::default(),
            grid: grid,
            steps: vec![],
        }
    }

    fn with_grid_format(grid: &'a Grid, format: &'a Table) -> Self {
        FindPaths {
            announce: Cell::new(silent),
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

fn silent(_: String) { }
// fn announce(x: String) { println!("{}", x); }
fn announce_fup(x: String) { println!("find_unclosed_path {}", x); }
fn announce_fupfe(x: String) { println!("find_unclosed_path fwd_ext {}", x); }
fn announce_fupre(x: String) { println!("find_unclosed_path rev_ext {}", x); }
fn announce_fuptre(x: String) { println!("find_unclosed_path try_rev_ext {}", x); }
fn announce_fcp(x: String) { println!("find_closed_path {}", x); }
fn announce_fcpf(x: String) { println!("find_closed_path_from {}", x); }

impl<'a> FindUnclosedPaths<'a> {
    fn find_unclosed_path(mut self, curr: Pt) -> Result<Path, Self> {
        self.find.check_inspection(curr, announce_fup);
        // start the search proper
        self.find.steps.push(curr);
        for (j, &dir) in DIRECTIONS.iter().enumerate() {
            debug!("find_unclosed_path {} dir: {:?}", j, dir);
            let next = DirVector(curr, dir).steps(1);
            debug!("find_unclosed_path {} dir: {:?} next: {:?}",
                   j, dir, next);
```

If we've gone off the grid, then obviously this direction is no good.
```rust
            if !self.find.grid.holds(next.0) {
                continue;
            }
```

We must ensure there is some Starting entry in the
format table for `curr` and `next`; otherwise, we will
not be able to start the rendering of the path.

```rust
            if !self.find.matches_start(curr, Some(next.0)) {
                continue;
            }

            match self.fwd_ext(next, FindContext { prev: Some(curr), curr: next.0 })
            {
                ret @ Ok(_) => return ret,
                Err(s) => self = s,
            }
        }
        debug!("find_unclosed_path self: {:?} exhausted directions; giving up.", self);
        return Err(self);
    }
```

Attempts to extends the end of the path forward via `dv`.
```rust
    fn fwd_ext(mut self, dv: DirVector, fc: FindContext) -> Result<Path, Self> {
        debug!("fwd_ext self: {:?} dv: {:?} {:?}", self, dv, fc);
        assert!(self.find.grid.holds(dv.0));
        assert!(!self.find.steps.contains(&dv.0));
        assert_eq!(dv.0, fc.curr());
        self.find.check_inspection(fc.curr(), announce_fupfe);
        let elem: Elem = self.find.grid[dv.0];
        debug!("fwd_ext elem: {:?}", elem);
        let _c: char = match elem {
            Elem::Pad | Elem::Clear => return Err(self), // blank: Give up.
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
            debug!("fwd_ext trying dir: {:?}", dir);
            let next = dv.towards(dir).step();

            if !self.find.grid.holds(next.0) { continue; } // off grid
            if self.find.steps.contains(&next.0) { continue; } // overlap
            if !self.find.matches(fc.prev, fc.curr, next.0) { // no format rule
                debug!("no format rule found for ({:?},{:?},{:?})",
                         fc.prev, fc.curr, next.0);
                continue;
            }


            match self.fwd_ext(next, FindContext { prev: Some(dv.0), curr: next.0 }) {
                p @ Ok(_) => return p,
                Err(s) => {
                    debug!("recursive search failed for ({:?},{:?},{:?})",
                             fc.prev, fc.curr, next.0);
                    self = s;
                    continue;
                }
            }
        }

        // If we get here, then none of the available directions worked, so attempt finish.
        assert_eq!(self.find.steps.last(), Some(&dv.0));
        if self.find.matches_end(fc.prev, fc.curr) {
            debug!("fwd_ext self: {:?} exhausted turns; finished.", self);

            Ok(self.rev_ext())
        } else {
            debug!("fwd_ext self: {:?} cannot end here; aborting.", self);
            // undo addition of `curr` to path
            assert_eq!(self.find.steps.last(), Some(&dv.0));
            self.find.steps.pop();
            Err(self)
        }
    }
}
```

These are a collection of methods related to reverse extension
of a path, i.e. finding a potential prefix after we have found
a suffix that is itself a legal path.

```rust
impl<'a> FindUnclosedPaths<'a> {
```

When we call this function, we have committed to the path that
we have found so far (i.e. the last item in the current reverse
prefix (if any) is a legal starting step for this path), and we
are trying to see if any futher prefix exists.

```rust
    fn rev_ext(mut self) -> Path {
        let (curr, next) = self.first_two();
        self.find.check_inspection(curr, announce_fupre);
        for (_j, &dir) in DIRECTIONS.iter().enumerate() {
            let prev = DirVector(curr, dir).steps(1);
```

Attempt to find a prefix following `dir`, returning it on success.

```rust
            match self.try_rev_ext(next, curr, prev) {
                Ok(p) => return p,
                Err(s) => self = s,
            }
        }
```

If we try all directions and none are usable, then just hand back
the path we have.

```rust
        self.to_path()
    }

    fn to_path(self) -> Path {
        let grid = self.find.grid;
        let mut steps = Vec::new();
        for pt in self.prefix_rev.into_iter().rev() {
            steps.push((pt, grid[pt].to_char()))
        }
        let mut p = self.find.to_path(Closed::Open);
        for step in p.steps {
            steps.push(step)
        }
        p.steps = steps;
        p
    }

    // Returns the first and second steps of the path.
    fn first_two(&self) -> (Pt, Pt) {
        match self.prefix_rev.len() {
            0 => (self.find.steps[0], self.find.steps[1]),
            n @ 1 => (self.prefix_rev[n-1], self.find.steps[0]),
            n => (self.prefix_rev[n-1], self.prefix_rev[n-2]),
        }
    }

    fn try_rev_ext(mut self, next: Pt, curr: Pt, prev: DirVector) -> Result<Path, Self> {
        self.find.check_inspection(curr, announce_fuptre);
```

If we've gone off the grid, then obviously this direction is no good.

```rust
        if !self.find.grid.holds(prev.0) {
            return Err(self);
        }
```

Likewise, if there is no content at this point, then we similarly
need not explore it.

```rust
        if self.find.grid[prev.0].is_blank() {
            return Err(self);
        }
```

When we call this function, we have a prefix+suffix that may not
be a legal path. So we need to:

1. Determine if prepending `prev` is legal at all. (If not, Err.)
   This involves:

   * checking if `prev` is not already on the prefix+suffix, and

```rust
        // if self.contains(prev.0) {
        if self.prefix_rev.contains(&prev.0) {
            return Err(self);
        }
        if self.find.steps.contains(&prev.0) {
            return Err(self);
        }
```

   * checking if `(prev, curr, next)` is an entry in our format table.

```rust
        if !self.find.matches(Some(prev.0), curr, next) {
            return Err(self);
        }
```

(At this point, we know that adding `prev` yields a legal suffix,
so tentatively add it to the `prefix_rev`.)

```rust
        self.prefix_rev.push(prev.0);
```

2. Determine if `prev` yields a valid start to the new prefix+suffix.
   (If so, then commit addition of `prev` by transitioning to `rev_ext`.)

```rust
        if self.is_start(prev.0, curr) {
            return Ok(self.rev_ext());
        }
```

3. Otherwise, prefix+suffix yields a suffix but not a whole path,
   loop through the directions and recursively call `try_rev_ext`
   on each and see if one works.

```rust
        let (curr, next) = (prev.0, curr);
        for (_j, &dir) in DIRECTIONS.iter().enumerate() {
            let prev = DirVector(curr, dir).steps(1);
            match self.try_rev_ext(next, curr, prev) {
                ret @ Ok(_) => return ret,
                Err(s) => self = s,
            }
        }
```

4. And if none of the directions work, then undo the addition
of `prev` and Err.

```rust
        assert_eq!(self.prefix_rev.last(), Some(&prev.0));
        self.prefix_rev.pop();

        return Err(self);
    }

    fn is_start(&self, curr: Pt, next: Pt) -> bool {
        self.find.matches_start(curr, Some(next))
    }
}

impl<'a> FindClosedPaths<'a> {
    fn is_corner(&self, curr: Pt) -> Option<Vec<((char, Direction), (Direction, char))>> {
        let grid = &self.find.grid;
        let c = match grid[curr] {
            Elem::C(c) | Elem::Used(c) => c,
            Elem::Pad | Elem::Clear => return None,
        };
        let mut in_out = Vec::new();
        for entry in &self.find.format.entries {
            if !entry.loop_start { continue; }
            if !entry.matches_curr(&self.find.announce.get(), c) { continue; }

            // This is a *potential* corner.
            //
            // check that there exist distinct neighbors that work
            // with the incoming and outgoing parts of the entry.
            let (in_match, in_dir) = entry.corner_incoming();
            let (out_dir, out_match) = entry.corner_outgoing();
            for in_dir in in_dir {
                for &out_dir in &out_dir {
                    if in_dir == out_dir { continue; }
                    let i = curr.neighbor(in_dir.reverse());
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

    fn find_closed_path(mut self, curr: Pt) -> Result<Path, Self> {
        self.find.check_inspection(curr, announce_fcp);
        let elem = self.find.grid[curr];
        // // Don't waste time on a search that starts on a non-corner.
        // // (all closed paths must have at least three corner elements,
        // //  since we need three points to define a positive 2D area;
        // //  so we can be assured that at least one corner exists
        // //  somewhere.)
        let corner_dirs: Vec<((char, Direction),
                              (Direction, char))> = match self.is_corner(curr) {
            None => return Err(self),
            Some(v) => v,
        };

        // start the search proper
        self.find.steps.push(curr);
        let mut out_dirs: Vec<Direction> = corner_dirs.iter().map(|t|(t.1).0).collect();
        out_dirs.sort();
        out_dirs.dedup();
        debug!("find_closed_path self: {:?} curr: {:?} pt: {:?} corner_dirs: {:?} out_dirs: {:?}",
               self, curr, elem, corner_dirs, out_dirs);
        for (j, &dir) in out_dirs.iter().enumerate() {
            let next = DirVector(curr, dir).steps(1);
            debug!("find_closed_path {} dir: {:?} next: {:?}",
                  j, dir, next);
            if !self.find.grid.holds(next.0) { // off grid
                continue;
            } else if next.0 == self.find.start() { // closes path
                panic!("this cannot happen so soon.");
            }
            match self.find_closed_path_from(next, FindLoopContext { prev: curr,
                                                                     curr: next.0,
                                                                     corner_dirs: &corner_dirs })
            {
                ret @ Ok(_) => return ret,
                Err(s) => self = s,
            }
        }
        debug!("find_closed_path self: {:?} exhausted directions; giving up.", self);
        return Err(self);
    }

    fn find_closed_path_from(mut self, dv: DirVector, fc: FindLoopContext) -> Result<Path, Self> {
        use directions::Turn;
        assert!(self.find.grid.holds(dv.0));
        assert!(!self.find.steps.contains(&dv.0));
        assert_eq!(dv.0, fc.curr);
        self.find.check_inspection(fc.curr, announce_fcpf);
        let elem: Elem = self.find.grid[dv.0];
        let c = match elem {
            Elem::Pad | Elem::Clear => return Err(self), // blank: Give up.
            Elem::C(c) | Elem::Used(c) => c
        };
        debug!("find_closed_path_from self: {:?} dv: {:?} fc: {:?} c: {:?}", self, dv, fc, c);

        self.find.steps.push(dv.0);

        // attempt to make the closed polygon via the sharpest
        // clockwise turn possible.
        let mut dir = dv.dir().sharp_turn(Turn::CW);
        // If that fails, we will try the next sharpest by veering
        // counter-clockwise, up until (but not including) we end up
        // going in the entirely reverse direction from where we
        // started.
        while dir != dv.dir().reverse() {
            let next = dv.towards(dir).step();
            if !self.find.grid.holds(next.0) {  // off grid
                dir = dir.veer(Turn::CCW);
                continue;
            }
            debug!("considering loop extension next: {:?} in {:?} fc: {:?}", next, self, fc);
            if self.find.start() == next.0  { // closes path; potential success!
                if self.properly_closed_loop(&fc) {
                    debug!("properly closed loop in {:?} fc: {:?}", self, fc);
                    return Ok(self.find.to_path(Closed::Closed));
                } else {
                    dir = dir.veer(Turn::CCW);
                    debug!("improperly closed in {:?} fc: {:?} veering to {:?}", self, fc, dir);
                    continue;
                }
            } else if self.find.steps.contains(&next.0) { // non-start overlap
                dir = dir.veer(Turn::CCW);
                continue;
            }
            if !self.find.matches(Some(fc.prev), fc.curr, next.0) { // no format rule
                dir = dir.veer(Turn::CCW);
                continue;
            }

            match self.find_closed_path_from(next, FindLoopContext { prev: dv.0,
                                                                     curr: next.0,
                                                                     ..fc }) {
                p @ Ok(_) => return p,
                Err(s) => {
                    self = s;
                    dir = dir.veer(Turn::CCW);
                    continue;
                }
            }
        }

        // If we get here, then none of the available directions
        // worked, so give up.
        assert_eq!(self.find.steps.last(), Some(&dv.0));
        self.find.steps.pop();
        debug!("find_closed_path self: {:?} exhausted turns; giving up.", self);
        return Err(self);
    }
}

impl<'a> FindClosedPaths<'a> {
    #[cfg(test)]
    fn new(grid: &'a Grid) -> Self {
        FindClosedPaths { find: FindPaths::new(grid) }
    }
    fn properly_closed_loop(&self, fc: &FindLoopContext) -> bool {
        let fst = self.find.steps[0];
        let snd = self.find.steps[1];
        let last = *self.find.steps.last().unwrap();
        let s = self.find.grid[snd].opt_char().unwrap();
        let l = self.find.grid[last].opt_char().unwrap();
        let finis_arc = last.towards(fst);
        let start_arc = fst.towards(snd);
        fc.corner_dirs.contains(&((l, finis_arc), (start_arc, s)))
    }
}

pub fn find_closed_path(grid: &Grid, format: &Table, pt: Pt) -> Option<Path> {
    let pf = FindClosedPaths { find: FindPaths::with_grid_format(grid, format) };
    let ret = pf.find_closed_path(pt).ok();
    debug!("find_closed_path pt {:?} ret {:?}", pt, ret);
    ret
}

pub fn find_unclosed_path_from(grid: &Grid, format: &Table, dir: DirVector) -> Option<Path> {
    let pf = FindUnclosedPaths { prefix_rev: Vec::new(),
                                 find: FindPaths::with_grid_format(grid, format) };
    let ret = pf.fwd_ext(dir, FindContext {
        prev: None,
        curr: dir.0
    }).ok();
    debug!("find_unclosed_path_from dir {:?} ret {:?}", dir, ret);
    ret
}

pub fn find_unclosed_path(grid: &Grid, format: &Table, pt: Pt) -> Option<Path> {
    let pf = FindUnclosedPaths { prefix_rev: Vec::new(),
                                 find: FindPaths::with_grid_format(grid, format) };
    let ret = pf.find_unclosed_path(pt).ok();
    debug!("find_unclosed_path pt {:?} ret {:?}", pt, ret);
    ret
}

impl<'a> FindPaths<'a> {
    fn check_inspection_start_at(&self, pt: Pt, a: fn (String)) {
        if !self.grid.holds(pt) { return; }
        if self.grid[pt].opt_char() == Some('%') {
            println!("TURNING ON ANNOUCER at {:?}", pt); 
            self.announce.set(a);
        }
    }

    fn check_inspection_finis_at(&self, pt: Pt) {
        if !self.grid.holds(pt) { return; }
        if self.grid[pt].opt_char() == Some('$') {
            println!("TURNING OFF ANNOUCER at {:?}", pt); 
            self.announce.set(silent);
        }
    }

    fn check_inspection_start(&self, curr: Pt, a: fn (String)) {
        self.check_inspection_start_at(curr, a);
        for (_j, &dir) in DIRECTIONS.iter().enumerate() {
            let nbor = DirVector(curr, dir).steps(1);
            self.check_inspection_start_at(nbor.0, a);
        }
    }

    fn check_inspection_finis(&self, curr: Pt) {
        self.check_inspection_finis_at(curr);
        for (_j, &dir) in DIRECTIONS.iter().enumerate() {
            let nbor = DirVector(curr, dir).steps(1);
            self.check_inspection_finis_at(nbor.0);
        }
    }

    fn check_inspection(&self, curr: Pt, a: fn (String)) {
        self.check_inspection_start(curr, a);
        self.check_inspection_finis(curr);
    }

    fn find_entry(&self,
                  prev: Option<Pt>,
                  curr: Pt,
                  next: Pt) -> Option<&format::Entry> {
        #![allow(unused_parens)]
        let c = (if let Some(c) = self.grid[curr].opt_char() { c }
                 else { return None; });
        let prev_arc = prev.and_then(|prev| {
            self.grid[prev].opt_char().map(|p| (p, prev.towards(curr)))
        });
        let next_arc = self.grid[next].opt_char().map(|n| (curr.towards(next), n));
        for entry in &self.format.entries {
            if entry.matches(&self.announce.get(), prev_arc, c, next_arc) {
                return Some(entry);
            }
        }
        return None;
    }

    fn matches(&self, prev: Option<Pt>, curr: Pt, next: Pt) -> bool {
        self.find_entry(prev, curr, next).is_some()
    }

    fn matches_start(&self, curr: Pt, next: Option<Pt>) -> bool {
        let c = if let Some(c) = self.grid[curr].opt_char() { c } else { return false; };
        let next_arc = next.and_then(|next| {
            self.grid[next].opt_char().map(|n| (curr.towards(next), n))
        });
        for entry in &self.format.entries {
            if entry.matches_start(&self.announce.get(), c, next_arc) {
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
            if entry.matches_end(&self.announce.get(), prev_arc, c) {
                return true;
            }
        }
        return false;
    }
}

#[cfg(test)]
mod tests;
```
