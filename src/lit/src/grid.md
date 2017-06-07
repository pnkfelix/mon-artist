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
/// A `Pt` is a trivial representation of a (row, column) in the grid.
///
/// Note that we use one-based indices when referring to row or column;
/// use the `fn row_idx` or `col_idx` methods to get 0-based indices.
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub struct Pt(pub i32, pub i32);

impl Pt {
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
pub struct PtRangeIterator {  }

pub trait PtRangeIter {
    fn iter(&self) -> PtRangeIterator;
}

use std::ops::RangeInclusive;

#[cfg(old)]
impl PtRangeIter for RangeInclusive<Pt> {
    fn iter(&self) -> PtRangeIterator {
        match *self {
            RangeInclusive::Empty { at: _ } => PtRangeIterator {
            },
            RangeInclusive::NonEmpty { start: _, end: _ } => PtRangeIterator {
            },
        }
    }
}

impl PtRangeIter for RangeInclusive<Pt> {
    fn iter(&self) -> PtRangeIterator {
        match *self {
            RangeInclusive { start: _, end: _ } => PtRangeIterator {
            },
        }
    }
}
```
