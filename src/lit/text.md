```rust
use grid::{Grid, Pt};

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct Text {
    pub (crate) pt: Pt,
    pub (crate) content: String,
    pub (crate) id: Option<(Pt, String)>,
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

    pub fn infer_id(&mut self, grid: &Grid) {
        let mut letter = self.pt;
        for _ in 0..self.content.len() {
            let pt = letter.s();
            if let Some(s) = grid.match_id(pt) {
                self.id = Some((pt, s));
            }
            letter = letter.e();
        }
    }
}

impl Text {
    pub fn attach_attributes(&mut self, pt: Pt, grid: &Grid) {
        if let Some((_, ref id)) = self.id {
            if let Some(attr) = grid.find_attr(id) {
                ::attrs::input_attr(&mut self.attrs, attr);
            }
        }
        if let Some(attr) = grid.find_pt_attr(pt) {
            ::attrs::input_attr(&mut self.attrs, attr);
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
        if let Some(ref pt_id) = t.id {
            self.clear_id(pt_id);
        }
    }
}
```
