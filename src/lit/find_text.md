```rust
use grid::{Grid, Pt};
use text::Text;

pub fn find_text(grid: &Grid, pt: Pt) -> Option<Text> {
    let mut tf = FindText::new(grid);
    tf.find_text(pt)
}

struct FindText<'a> {
    grid: &'a Grid,
}

impl<'a> FindText<'a> {
    pub fn new(grid: &Grid) -> FindText {
        FindText { grid: grid }
    }

    pub fn find_text(&mut self, start: Pt) -> Option<Text> {
        use grid::Elem::{Clear, Pad, Used, C};
        let grid = self.grid;
        assert!(grid.holds(start));
        let elem = grid[start];
        let mut c = match elem {
            Clear | Pad | Used(..) | C(' ') => return None,
            C(c) => c,
        };
        let grid = self.grid;
        let mut buf = String::new();
        let mut curr = start;
        loop {
            buf.push(c);
            curr = curr.e();
            if !grid.holds(curr) { return Some(Text::new(start, buf)); }
            let elem = grid[curr];
            c = match elem {
                Clear | Pad | Used(..) => return Some(Text::new(start, buf)),
                C(c) => c,
            };
        }
    }
}
```
