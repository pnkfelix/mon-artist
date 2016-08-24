The `asciitosvg` Rust library is inspired by PHP library of the same name,
aka [a2s][].

[a2s]: https://9vx.org/~dho/a2s/

The goal is to render ASCII art diagrams into SVG. Just like [a2s][],
we only attempt to do nice conversions on a restricted subset of
arbitrary ASCII art, but we attempt to render it in a way that
maintains the relative proportions of the original text.

This means that if you have a particular layout in your ASCII art,
such as positioning objects to be lined up with each other, that
layout will be maintained in the generated picture.

## The `lib` root

### Features


```rust
#![feature(pub_restricted)]
#![feature(inclusive_range_syntax, inclusive_range)]
#![feature(type_ascription)]
#![feature(slice_patterns)]
```


### External crates

Most of the content will be interpreted by hand-written routines traversing
a character grid, but there is one exception: the markdown-style
`[ident]` forms that can occur at the end of an input diagram.
While I could have made a hand-written parser for that, it makes
more sense to just use a regexp, i.e. the `regex` crate.
And efficient use of regexps requires defining them via the
`lazy_static!` macro; see [regex docs][]

[regex docs]: https://doc.rust-lang.org/regex/regex/index.html#example-avoid-compiling-the-same-regex-in-a-loop

```rust
#[macro_use] extern crate lazy_static;
extern crate regex;
```

Since the output format is SVG, it makes some amount of sense for us to
build up the output as an XML document. (It might also work to go direct
to text output, which is what [a2s][] does.)

Building up an XML document means choosing a representation for such documents.
`treexml` is the first thing I found on [crates.io][] that seemed to fit.

```rust
extern crate treexml;
```

TODO say something about logging here.
At the very least, note that one needs to write `::env_logger::init();` explicitly in unit tests when one wants that.

```rust
#[macro_use]
extern crate log;
extern crate env_logger;
```

Once we have the external crates declared, we can jump into our own
definitions.

First we have basic modules.

The `directions` module defines the eight compass directions one can
traverse from a point on the grid. It defines them both in a single
enum `Direction`, as well as in corresponding static types (for use
for trait trickery when encoding data).
```rust
pub mod directions;
```

The `grid` module defines the core `Grid` data type: it represents the
grid of characters that we initially read in, as well as intermediate
states of that grid as it is repeatedly processed in our attempt to
derive a high-level picture from it.

```rust
pub mod grid;
```

The `attrs` module is a grab bag of utility functions for parsing
and manipulating the key=value attributes that one sees in XML
elements.

```rust
mod attrs {
    use regex::Regex;

    fn split_attr(attr: &str) -> Vec<(String, String)> {
        lazy_static! {
            static ref KV: Regex =
                Regex::new(r#"([a-zA-Z0-9_-]*)='([^']*)'|([a-zA-Z0-9_-]*)="([^"]*)""#)
                .unwrap_or_else(|e| panic!("ill-formatted regex: {}", e));
        }

        let mut attrs = Vec::new();
        for cap in KV.captures_iter(attr) {
            debug!("cap: '{:?}' cap.at(0): '{:?}'", cap, cap.at(0));
            attrs.push((cap.at(1).unwrap().to_string(),
                        cap.at(2).unwrap().to_string()));
        }
        attrs
    }

    pub fn input_attr(attrs: &mut Option<Vec<(String, String)>>, attr: &str) {
        let new = split_attr(attr);
        if new.is_empty() { return; }
        match *attrs {
            Some(ref mut attrs) => {
                attrs.extend(new);
            }
            None => {
                *attrs = Some(new);
            }
        }
    }
}
```

The `svg` module defines the interface for building up the output SVG.

```rust
pub mod svg;
```

The `path` module represents the paths that we extract by following
adjacent characters on a grid. These paths can take the form of closed
polygons, or lines, which are all the fragments of polygons that we
could not close).

```rust
pub mod path;
pub use path::Path;

pub mod find_path;
```

The `text` and `find_text` modules are analogous to `path` and `find_path`,
except they search for blocks of `<text>` rather than paths to draw.

```rust
pub mod text;
pub mod find_text;
```

Finding paths on a grid yields a "scene", which holds the
paths themselves and the stage on which they are drawn.

Since every scene comes from an ASCII art grid, we measure
abstractly of the number of horizontal and vertical "elements"
that appear on the scene. Note that the width of an individual
element may not equal its height; e.g. in nearly all fixed-width
fonts, each character occupies more space vertically than
horizontally.

```rust
mod scene {
    use path::{Path};
    use grid::{Grid};
    use text::{Text};

    pub struct Scene {
        paths: Vec<Path>,
        texts: Vec<Text>,
        /// The number of elements that need to fit horizontally in the scene.
        width: u32,
        height: u32,
    }

    impl Scene {
        pub fn paths(&self) -> &[Path] { &self.paths }
        pub fn texts(&self) -> &[Text] { &self.texts }
        pub fn width(&self) -> u32 { self.width }
        pub fn height(&self) -> u32 { self.height }
    }

    impl Grid {
        pub fn into_scene(mut self) -> Scene {
            use find_path::{find_closed_path, find_unclosed_path};
            use find_text::{find_text};
            use grid::{Pt};
            let mut paths = vec![];
            let mut texts = vec![];
            for row in 1...self.height {
                for col in 1...self.width {
                    loop {
                        let pt = Pt(col as i32, row as i32);
                        if let Some(mut p) = find_closed_path(&self, pt) {
                            p.infer_id(&self);
                            p.attach_attributes(pt, &self);
                            debug!("pt {:?} => closed path {:?}", pt, p);
                            self.remove_path(&p);
                            paths.push(p);
                        } else {
                            break;
                        }
                    }
                }
            }
            for row in 1...self.height {
                for col in 1...self.width {
                    loop {
                        let pt = Pt(col as i32, row as i32);
                        if let Some(mut p) = find_unclosed_path(&self, pt) {
                            p.infer_id(&self);
                            p.attach_attributes(pt, &self);
                            debug!("pt {:?} => unclosed path {:?}", pt, p);
                            self.remove_path(&p);
                            paths.push(p);
                        } else {
                            break;
                        }
                    }
                }
            }
            for row in 1...self.height {
                for col in 1...self.width {
                    loop {
                        let pt = Pt(col as i32, row as i32);
                        if let Some(mut txt) = find_text(&self, pt) {
                            txt.infer_id(&self);
                            txt.attach_attributes(pt, &self);
                            debug!("txt {:?} => text {:?}", pt, txt);
                            self.remove_text(&txt);
                            texts.push(txt);
                        } else {
                            break;
                        }
                    }
                }
            }
            Scene { paths: paths, texts: texts, width: self.width, height: self.height }
        }
    }
}
pub use scene::Scene;

use treexml::{Element};
use svg::{ToElement};

impl ToElement for Scene {
    fn to_element(&self) -> Element {
        let mut s = String::new();
        let mut e = Element::new("pre");
        e.attributes.insert("style".to_string(),
                            "font-size: 50%;".to_string());
        s.push_str(&format!("width: {} height: {}\n", self.width(), self.height()));
        for (i, p) in self.paths().iter().enumerate() {
            s.push_str(&format!("path[{}]: Path {{ id: {:?}, closed: {:?}, attrs: {:?}\n",
                                i, p.id, p.closed, p.attrs));
            for (j, step) in p.steps.iter().enumerate() {
                s.push_str(&format!("    path[{}][{}]: {:?},\n", i, j, step));
            }
            s.push_str(&"}\n");
        }
        for (i, t) in self.texts().iter().enumerate() {
            s.push_str(&format!("text[{}]: Text {{ \
                                     id: {:?} pt: ({},{}), content: {}, attrs: {:?} \
                                 }}\n",
                                i, t.id, t.pt.col(), t.pt.row(), t.content, t.attrs));
        }
        e.text = Some(s);
        e
    }
}
```

The `test_data` module holds various examples input grids, used for
writing unit tests of routines as I write them.

```rust
pub mod test_data;
```

The `format` module handles the user-customizable
formatting description. It currently is tailored to SVG
descriptions and only affects the rendering step, but
eventually I want it to drive the input parsing as well
(allowing for arbitrary unicode to be used in interesting ways).

```rust
pub mod format;
```

The `render` module holds the core routines for rendering grids.

```rust
pub mod render;

#[test]
fn end_to_end_basics() {
    const PRINT_INTERMEDIATE_SCENE_STRUCTURE: bool = false;

    use grid::{Grid};
    use render::{RenderS};
    use render::svg::{SvgRender};
    use svg::{IntoElement, ToElement};
    use treexml::{Document, Element};
    use std::path::{Path};
    use std::fs::{File};
    use std::io::{Write};
    let _ = ::env_logger::init();
    let mut html_doc = Document::default();
    let mut html_body = Element::new("body");
    for &(name, d) in &test_data::ALL {
        let r = SvgRender {
            x_scale: 8, y_scale: 13,
            font_family: "Menlo".to_string(), font_size: 13,
            show_gridlines: true,
            name: name.to_string(),
        };
        html_body.children.push({
            let mut e = Element::new("h3");
            e.text = Some(name.to_string());
            e
        });
        html_body.children.push({
            let mut e = Element::new("pre");
            e.attributes.insert("style".to_string(),
                                "border:1px dotted black;".to_string());
            e.text = Some(format!("{}", d));
            e
        });
        let s = d.parse::<Grid>().unwrap().into_scene();

        if PRINT_INTERMEDIATE_SCENE_STRUCTURE {
            html_body.children.push(s.to_element());
        }

        let elem = r.render_s(&s);
        html_body.children.push(elem.into_element());
        html_body.children.push(Element::new("hr"));
    }
    let mut html_elem = Element::new("html");
    html_elem.children.push(html_body);
    html_doc.root = Some(html_elem);
    let mut f = File::create(Path::new("basics.html")).unwrap();
    write!(&mut f, "{}", html_doc).unwrap();
}
```
