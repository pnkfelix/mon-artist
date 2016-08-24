This module will define the representation for our grid of characters.

```rust
use directions::{Direction, Turn};
```

A `Pt` represents a point in 2d space; it may or may not fall on the grid.

Regarding the choice of `usize` vs `u32` vs `i32`:

 * since the grids are always going to be input texts, it is safe to
   assume that they will be well under `i32::MAX` in width or height.

 * furthermore, some of the trajectory calculations may involve points
   that fall off the grid. (This is unlikely to occur in practice, but
   I see no reason to rule it out from the outset.)  Handling that
   case is easier if we allow the x/y dimensions of a point to be
   negative (it is just another case of falling outside the bounds of
   the grid).


```rust
use regex::Regex;
use std::str::FromStr;

/// A `Pt` is a trivial representation of a (row, column) in the grid.
///
/// Note that we use one-based indices when referring to row or column;
/// use the `fn row_idx` or `col_idx` methods to get 0-based indices.
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub struct Pt(pub i32, pub i32);

impl Pt {
    pub fn col(&self) -> i32 { self.0 }
    pub fn row(&self) -> i32 { self.1 }
    pub fn row_idx(&self) -> usize { (self.row() - 1) as usize }
    pub fn col_idx(&self) -> usize { (self.col() - 1) as usize }
    pub fn shift(&self, dcol: i32, drow: i32) -> Pt {
        Pt(self.0 + dcol, self.1 + drow)
    }
    pub fn e(&self) -> Pt { self.shift(1,0) }
    pub fn n(&self) -> Pt { self.shift(0,-1) }
    pub fn s(&self) -> Pt { self.shift(0,1) }
    pub fn w(&self) -> Pt { self.shift(-1,0) }

    pub fn towards(&self, target: Pt) -> Direction {
        let dcol = target.col() - self.col();
        let drow = target.row() - self.row();

        // basic sanity check: ensure we are on one of the eight compass points.
        assert!(dcol.abs() == drow.abs() || // NE,SE,SW,NW
                dcol == 0 || // N,S
                drow == 0); // E,W

        let southern = drow > 0;
        let eastern = dcol > 0;
        let western = dcol < 0;
        let northern = drow < 0;

        match (northern, eastern, western, southern) {
            (false, false, false, false) => panic!("towards: given Pts must be distinct"),
            (true, false, false, _) => Direction::N,
            (true, true, _, _) => Direction::NE,
            (true, _, true, _) => Direction::NW,
            (_, false, false, true) => Direction::S,
            (_, true, _, true) => Direction::SE,
            (_, _, true, true) => Direction::SW,
            (false, true, _, false) => Direction::E,
            (false, _, true, false) => Direction::W,
        }
    }
}
```

TODO: document how a Pt supports IntoIterator to ease iterator chaining with singletons.

```rust
impl IntoIterator for Pt {
    type Item = Pt;
    type IntoIter = ::std::option::IntoIter<Pt>;
    fn into_iter(self) -> Self::IntoIter { Some(self).into_iter() }
}

pub trait PtCharIntoIterator {
    type IntoIter: Iterator<Item=(Pt, char)>;
    fn into_iter(self) -> Self::IntoIter;
}

impl PtCharIntoIterator for (Pt, char) {
    type IntoIter = ::std::option::IntoIter<(Pt,char)>;
    fn into_iter(self) -> Self::IntoIter { Some(self).into_iter() }
}
```

TODO: document how a Range of Pts yields Iterators

```rust
pub struct PtRangeIterator { dv: DirVector, limit: Pt, incl: bool, done: bool }
pub struct PtCharRangeIterator(PtRangeIterator, char);

pub trait PtRangeIter {
    fn iter(&self) -> PtRangeIterator;
    fn iter_char(&self, c: char) -> PtCharRangeIterator {
        PtCharRangeIterator(self.iter(), c)
    }
}

use std::ops::RangeInclusive;

impl PtRangeIter for RangeInclusive<Pt> {
    fn iter(&self) -> PtRangeIterator {
        match *self {
            RangeInclusive::Empty { at } => PtRangeIterator {
                dv: DirVector(at, Direction::N),
                limit: at,
                incl: true,
                done: true,
            },
            RangeInclusive::NonEmpty { start, end } => PtRangeIterator {
                dv: DirVector(start, start.towards(end)),
                limit: end,
                incl: true,
                done: false,
            },
        }
    }
}

impl PtRangeIter for ::std::ops::Range<Pt> {
    fn iter(&self) -> PtRangeIterator {
        PtRangeIterator {
            dv: DirVector(self.start, self.start.towards(self.end)),
            limit: self.end,
            incl: false,
            done: false,
        }
    }
}

impl Iterator for PtRangeIterator {
    type Item = Pt;
    fn next(&mut self) -> Option<Pt> {
        if self.done { return None; }
        let ret = self.dv.pt();
        if ret == self.limit { self.done = true; return if self.incl { Some(ret) } else { None }; }
        self.dv = self.dv.step();
        return Some(ret);
    }
}

impl Iterator for PtCharRangeIterator {
    type Item = (Pt, char);
    fn next(&mut self) -> Option<(Pt, char)> {
        self.0.next().map(|p| (p, self.1))
    }
}

impl Pt {
    pub fn rowcol(r: i32, c: i32) -> Pt { Pt(c, r) }
    pub fn colrow(c: i32, r: i32) -> Pt { Pt(c, r) }

    pub fn neighbor(&self, dir: Direction) -> Pt {
        Pt::rowcol((self.row() + dir.ver_south()),
                   (self.col() + dir.hor_east()))
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct DirVector(pub Pt, pub Direction);

impl DirVector {
    pub fn pt(&self) -> Pt { self.0 }
    pub fn dir(&self) -> Direction { self.1 }
    pub fn shift(&self, drow: i32, dcol: i32) -> DirVector {
        DirVector(self.pt().shift(drow, dcol), self.dir())
    }
    pub fn towards(&self, dir: Direction) -> DirVector {
        DirVector(self.pt(), dir)
    }
    pub fn veer(&self, turn: Turn) -> DirVector {
        DirVector(self.pt(), self.dir().veer(turn))
    }
    pub fn sharp_turn(&self, turn: Turn) -> DirVector {
        DirVector(self.pt(), self.dir().sharp_turn(turn))
    }
    pub fn step(&self) -> DirVector { self.steps(1) }
    pub fn steps(&self, mut count: u32) -> DirVector {
        let DirVector(mut pt, dir) = *self;
        while count > 0 {
            pt = pt.neighbor(dir);
            count -= 1;
        }
        DirVector(pt, dir)
    }
}
/// An element on the `Grid`.
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub enum Elem {
    /// A character from the original input that has not yet been used in any path construction.
    C(char),

    /// A character from the original input that was used in some path construction.
    Used(char),

    /// Padding inserted to make all the rows have the same
    /// length.
    Pad,

    /// Marker indicating that we already removed the character
    /// that fell above this.
    Clear,
}

impl Elem {
    pub fn to_char(&self) -> char {
        match *self {
            Elem::C(c) | Elem::Used(c) => c,
            Elem::Pad => ' ',
            Elem::Clear => '\u{7f}',
        }
    }

    pub fn is_line(&self) -> bool {
        match *self {
            Elem::C('-') |
            Elem::C('=') |
            Elem::C('|') |
            Elem::C(':') |
            Elem::C('/') |
            Elem::C('\\') => true,
            _ => false,
        }
    }

    pub fn is_corner(&self) -> bool {
        match *self {
            Elem::C('.') |
            Elem::C('\'') |
            Elem::C('+') => true,
            _ => false,
        }
    }

    pub fn is_used(&self) -> bool {
        match *self {
            Elem::Used(_) => true,
            _ => false,
        }
    }

    pub fn is_blank(&self) -> bool {
        match *self {
            Elem::C(' ') | Elem::C('\u{7f}') |
            Elem::Pad |
            Elem::Clear =>  true,
            _ => false,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct Grid {
    pub rows: Vec<Vec<Elem>>,
    pub width: u32,
    pub height: u32,
    pub attrs: Vec<(String, String)>,
}

impl Grid {
    pub fn holds(&self, pt: Pt) -> bool {
        pt.col() >= 1 &&
            pt.row() >= 1 &&
            pt.row() as u32 <= self.height &&
            pt.col() as u32 <= self.width
    }

    fn assert_pt_valid(&self, pt: Pt) {
        // we use one-based indexing in Pt, so subtract 1 to convert to zero-based indexing.
        assert!(pt.row() >= 1);
        assert!(pt.row_idx() < self.rows.len(), "row_idx: {} len: {}",
                pt.row_idx(), self.rows.len());
        assert!(pt.col_idx() < self.rows[pt.row_idx()].len());
    }

    pub fn set(&mut self, pt: Pt, value: Elem) {
        self.assert_pt_valid(pt);
        self.rows[pt.row_idx()][pt.col_idx()] = value;
    }

    pub fn mark_used(&mut self, pt: Pt) {
        self.assert_pt_valid(pt);
        let elem = self[pt];
        let new_elem = match elem {
            Elem::C(c) | Elem::Used(c) => Elem::Used(c),
            Elem::Pad | Elem::Clear => panic!("cannot mark clear spot as used."),
        };
        self.set(pt, new_elem);
    }
}

use std::ops::Index;

impl Index<Pt> for Grid {
    type Output = Elem;
    fn index(&self, pt: Pt) -> &Elem {
        self.assert_pt_valid(pt);
        &self.rows[pt.row_idx()][pt.col_idx()]
    }
}

#[derive(Debug)]
pub enum ParseError {
}

impl FromStr for Grid {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Grid, ParseError> {
        use std::u32;

        lazy_static! {
            static ref MD_RE: Regex = Regex::new(r"^\[([^\]\n]+)\]: (.*)")
                .unwrap_or_else(|e| panic!("ill-formatted regex: {}", e));
        }

        let lines = s.lines();
        let mut rows: Vec<Vec<Elem>> = lines
            .take_while(|line| !MD_RE.is_match(line))
            .map(|line| line.chars().map(|c| Elem::C(c)).collect())
            .collect();
        let height = rows.len();
        let width = rows.iter().map(|row| row.len()).max().unwrap_or(0);
        assert!(width <= u32::MAX as usize);
        assert!(height <= u32::MAX as usize);

        let attrs = s.lines()
            .skip(height)
            .filter_map(|line| {
                MD_RE.captures(line)
                    .map(|caps| {
                        (caps.at(1).unwrap().to_string(), caps.at(2).unwrap().to_string())
                    })
            })
            .collect();
        for row in &mut rows {
            row.resize(width, Elem::Pad);
        }
        let width = width as u32;
        let height = height as u32;
        Ok(Grid { rows: rows, width: width, height: height, attrs: attrs, })
    }
}

impl Grid {
    pub fn match_id(&self, pt: Pt) -> Option<String> {
        if !self.holds(pt) { return None; }
        if self[pt] != Elem::C('[') { return None }
        let mut pt = pt;
        let mut id = String::new();
        loop {
            pt = pt.e();
            if !self.holds(pt) { return None; }
            match self[pt] {
                Elem::C(']') => { if id.len() > 0 { return Some(id) } else { return None; } }
                Elem::C(c) => { id.push(c); }
                _ => return None,
            }
        }
    }

    pub fn clear_id(&mut self, &(pt, ref id): &(Pt, String)) {
        let r = pt.row();
        let c = pt.col();
        let len = id.chars().count();
        for c_i in c..(c+2+(len as i32)) {
            self.set(Pt::rowcol(r,c_i), Elem::Clear);
        }
    }

    pub fn mark_id_as_used(&mut self, &(pt, ref id): &(Pt, String)) {
        let r = pt.row();
        let c = pt.col();
        let len = id.chars().count();
        for c_i in c..(c+2+(len as i32)) {
            self.mark_used(Pt::rowcol(r,c_i));
        }
    }
}

impl Grid {
    pub fn to_string(&self) -> String {
        let mut s = String::new();
        for row in &self.rows {
            let mut saw_pad = false;
            for e in row {
                match *e {
                    Elem::C(c) => {
                        assert!(!saw_pad);
                        s.push(c);
                    }
                    Elem::Pad => {
                        saw_pad = true;
                    }
                    Elem::Used(c) => {
                        assert!(!saw_pad);
                        s.push(c);
                    }
                    Elem::Clear => {
                        assert!(!saw_pad);
                        s.push('_');
                    }
                }
            }
            s.push('\n');
        }
        for &(ref key, ref val) in &self.attrs {
            s.push_str(&format!("[{}]: {}\n", key, val));
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::Grid;
    use test_data;

    #[test]
    fn basics() {
        let grid: Grid = test_data::BASIC.1.parse().unwrap();
        assert_eq!(grid.height, test_data::BASIC_HEIGHT);
        assert_eq!(grid.width, test_data::BASIC_WIDTH);
        for row in &grid.rows { assert_eq!(row.len(), grid.width as usize); }
        assert_eq!(grid.to_string(), test_data::BASIC.1);
    }

    #[test]
    fn basic_attrs() {
        let grid: Grid = test_data::BASIC_ATTRS.1.parse().unwrap();
        assert_eq!(grid.height, test_data::BASIC_HEIGHT);
        assert_eq!(grid.width, test_data::BASIC_WIDTH);
        for row in &grid.rows { assert_eq!(row.len(), grid.width as usize); }
        assert_eq!(grid.to_string(), test_data::BASIC_ATTRS.1);
    }

    #[test]
    fn issue_15() {
        let grid: Grid = test_data::ISSUE_15_DESC.1.parse().unwrap();
        assert_eq!(grid.to_string(), test_data::ISSUE_15_DESC.1);
    }
}
```
