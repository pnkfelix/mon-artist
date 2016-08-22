```rust
use grid::{Grid, Pt};

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct Text {
    pub (crate) pt: Pt,
    pub (crate) content: String,
    pub (crate) id: Option<String>,
    pub (crate) attrs: Option<Vec<(String, String)>>,
}

impl Text {
    pub fn new(pt: Pt, content: String) -> Text {
        Text {
            pt: pt,
            content: content,
            id: None,
            attrs: None,
        }
    }
}

impl Grid {
    pub fn remove_text(&mut self, t: &Text) {
        let r = t.pt.row();
        let c = t.pt.col();
        let len = t.content.chars().count();
        for c_i in c..c+(len as i32) {
            self.mark_used(Pt::rowcol(r,c_i));
        }
    }
}
```
