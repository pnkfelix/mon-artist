```rust
use treexml::Element;

trait InsertAttribute {
    fn insert_attribute<S1, S2>(&mut self, attr: S1, val: S2)
        where S1: Into<String>, S2: Into<String>;
}

impl InsertAttribute for Element {
    fn insert_attribute<S1, S2>(&mut self, attr: S1, val: S2)
        where S1: Into<String>, S2: Into<String>
    {
        self.attributes.insert(attr.into(), val.into());
    }
}
```

A silly numeric representation.
For `Dec(a,b)`, `a` is the integral part and `b` is the fractional part,
in the sense that the printed string form of the number is `a.b`.

(Basically this makes reading and writing literal forms very easy
but makes every other aspect of arithmetic hard.)

Why is it silly? Well, beyond things like making arithmetic hard,
it also conflates values; e.g. a fractional part of `1` is the same
as one of `10`, and `100`, et cetera.

Even worse, one cannot represent `1.01` with this data type as
currently defined. (That might be the killer issue for me, in
fact, because I am worried about such intermediate values arising
in practice.  I probably will switch to a different representation
and try to just add `Dec(a,b)` as a constructor function.

```rust
#[derive(Copy, Clone, PartialEq, Eq)]
struct Dec(u32, u32);

pub fn correct(frac: u32, new_frac: u32) -> (u32, u32) {
    let denom = denom(frac);
    let carry = new_frac / denom;
    let rem = new_frac - (carry * denom);
    (carry, rem)
}

fn denom(frac: u32) -> u32 {
    match frac {
        0       ...9        => 10,
        10      ...99       => 100,
        100     ...999      => 1000,
        1000    ...9999     => 10000,
        10000   ...99999    => 100000,
        100000  ...999999   => 1000000,
        1000000 ...9999999  => 10000000,
        10000000...99999999 => 100000000,
        _ => unimplemented!(),
    }
}

impl Dec {
    fn scale(&self, scale: u32) -> Dec {
        let Dec(nat, frac) = *self;
        // if frac == 0 { return (nat * scale, 0); }
        let (carry, rem) = correct(frac, frac * scale);
        Dec(nat * scale + carry, rem)
    }

    #[allow(dead_code)]
    fn add_half(&self) -> Dec {
        let Dec(nat, frac) = *self;
        let denom = denom(frac) / 10;
        let (carry, rem) = correct(frac, frac + 5 * denom);
        Dec(nat + carry, rem)
    }

    fn sub_half(&self) -> Dec {
        let Dec(nat, frac) = *self;
        let denom = denom(frac) / 10;
        if frac >= 5 * denom {
            Dec(nat, frac - 5 * denom)
        } else {
            assert!(nat >= 1, "Dec {}.{} sub_half() failed", nat, frac);
            Dec(nat - 1, 5 * denom + frac)
        }
    }
}

/// `Dim` is a measure of dimension. They are meant to carry units,
/// though it can be omitted via the `U` variant.
#[derive(Copy, Clone, PartialEq, Eq)] // if we ever impl PartialOrd, do it explicitly.
pub enum Dim {
    /// Unit-less; e.g. U(50,1) is "50.1"
    U(u32, u32),
    /// Pixel count; e.g. Px(50) is "50px"
    Px(u32),
    /// Pct stands for "Percent". e.g. Pc(50) is "50%"
    Pc(u32, u32),
}

impl Default for Dim {
    fn default() -> Self { Dim::U(0, 0) }
}

impl Dim {
    pub fn to_string(&self) -> String {
        match *self {
            Dim::U(s, 0) => format!("{}", s),
            Dim::U(n, d) => format!("{}.{}", n, d),
            Dim::Px(s) => format!("{}px", s),
            Dim::Pc(s, 0) => format!("{}%", s),
            Dim::Pc(n, d) => format!("{}.{}%", n, d),
        }
    }

    fn to_dec(&self) -> Dec {
        match *self {
            Dim::U(s, f) => Dec(s, f),
            Dim::Px(_) => panic!("should never convert pixel dim to Dec"),
            Dim::Pc(s, f) => Dec(s, f),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn add_half(&self) -> Dim {
        match *self {
            Dim::U(..) => self.to_dec().add_half().to_u(),
            Dim::Px(_) => panic!("should never add half to pixel dim"),
            Dim::Pc(..) => self.to_dec().add_half().to_pc(),
        }
    }

    pub(crate) fn sub_half(&self) -> Dim {
        match *self {
            Dim::U(..) => self.to_dec().sub_half().to_u(),
            Dim::Px(_) => panic!("should never sub half from pixel dim"),
            Dim::Pc(..) => self.to_dec().sub_half().to_pc(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn sub_one(&self) -> Dim {
        self.sub_half().sub_half()
    }

    #[allow(dead_code)]
    pub(crate) fn div_2(&self) -> Dim {
        match *self {
            Dim::U(n,0) if n % 2 == 0 => Dim::U(n/2,0),
            Dim::U(n,0) if n % 2 == 1 => Dim::U(n/2,5),
            Dim::Pc(n,0) if n % 2 == 0 => Dim::Pc(n/2,0),
            Dim::Pc(n,0) if n % 2 == 1 => Dim::Pc(n/2,5),
            _ => unimplemented!(),
        }
    }
}

trait ToDimU { fn to_u(&self) -> Dim; }
trait ToDimPx {  fn to_px(&self) -> Dim; }
trait ToDimPc { fn to_pc(&self) -> Dim; }

impl ToDimU  for (u32, u32) { fn to_u(&self) -> Dim { Dim::U(self.0, self.1) } }
impl ToDimU  for Dec { fn to_u(&self) -> Dim { Dim::U(self.0, self.1) } }
impl ToDimU  for u32        { fn to_u(&self) -> Dim { Dim::U(*self, 0) } }
impl ToDimPx for u32        { fn to_px(&self) -> Dim { Dim::Px(*self) } }
impl ToDimPc for (u32, u32) { fn to_pc(&self) -> Dim { Dim::Pc(self.0, self.1) } }
impl ToDimPc for Dec { fn to_pc(&self) -> Dim { Dim::Pc(self.0, self.1) } }

use std::ops::Mul;
impl Mul<u32> for Dim {
    type Output = Dim;
    fn mul(self, rhs: u32) -> Dim {
        match self {
            Dim::U(..)  => self.to_dec().scale(rhs).to_u(),
            Dim::Px(n)   => (n*rhs).to_px(),
            Dim::Pc(..) => self.to_dec().scale(rhs).to_pc(),
        }
    }
}

mod color;
pub use self::color::Color;

#[derive(Clone, PartialEq, Eq)]
pub enum Fill {
    Color(Color),
    Pattern { def_id: String },
    None,
}

impl Fill {
    fn into_string(self) -> String {
        match self {
            Fill::Color(c) => c.into_string(),
            Fill::Pattern { def_id } => format!("url(#{})", def_id),
            Fill::None => "None".to_string()
        }
    }
}

pub struct Rect {
    pub x: Dim,
    pub y: Dim,
    pub width: Dim,
    pub height: Dim,
    pub fill: Fill,
    pub stroke: Option<(Fill, Dim)>,
    pub rounded: Option<(Dim, Dim)>,
}

pub trait IntoElement {
    fn into_element(self) -> Element;
}

impl IntoElement for Rect {
    fn into_element(self) -> Element {
        let mut e = Element::new("rect");
        let Rect { x, y, width, height, fill, stroke, rounded } = self;
        e.insert_attribute("x", x.to_string());
        e.insert_attribute("y", y.to_string());
        e.insert_attribute("width", width.to_string());
        e.insert_attribute("height", height.to_string());
        e.insert_attribute("fill", fill.into_string());
        if let Some((stroke, stroke_width)) = stroke {
            e.insert_attribute("stroke", stroke.into_string());
            e.insert_attribute("stroke-width", stroke_width.to_string());
        }
        if let Some((rx, ry)) = rounded {
            e.insert_attribute("rx", rx.to_string());
            e.insert_attribute("ry", ry.to_string());
        }
        e
    }
}

pub struct Circle { pub cx: Dim, pub cy: Dim, pub r: Dim, pub fill: Color }

impl IntoElement for Circle {
    fn into_element(self) -> Element {
        let mut e = Element::new("circle");
        e.insert_attribute("cx", self.cx.to_string());
        e.insert_attribute("cy", self.cy.to_string());
        e.insert_attribute("r", self.r.to_string());
        e.insert_attribute("fill", self.fill.into_string());
        e
    }
}

pub use self::text::Text;

impl IntoElement for Text {
    fn into_element(self) -> Element {
        let mut e = Element::new("text");
        e.insert_attribute("x", self.x.to_string());
        e.insert_attribute("y", self.y.to_string());
        e.insert_attribute("font-size", self.font_size.to_string());
        e.insert_attribute("text-anchor", self.text_anchor.to_string());
        e.insert_attribute("fill", self.fill.into_string());
        e
    }
}

pub struct Path { pub d: String, pub attrs: Vec<(String, String)> }

impl IntoElement for Path {
    fn into_element(self) -> Element {
        let mut e = Element::new("path");
        e.insert_attribute("d", self.d);
        for (k, v) in self.attrs {
            e.insert_attribute(k, v);
        }
        e
    }
}

pub mod text {
    use super::{Color, Dim};

    pub enum TextAnchor {
        Middle,
    }

    impl TextAnchor {
        pub fn to_string(&self) -> String {
            match *self {
                TextAnchor::Middle => "middle".to_string(),
            }
        }
    }

    pub struct Text {
        pub x: Dim,
        pub y: Dim,
        pub font_size: Dim,
        pub text_anchor: TextAnchor,
        pub fill: Color,
        pub content: String,
    }
}

pub enum Shape {
    Rect(Rect),
    Circle(Circle),
    Text(Text),
    Path(Path),
}

impl IntoElement for Shape {
    fn into_element(self) -> Element {
        match self {
            Shape::Rect(r) => r.into_element(),
            Shape::Circle(c) => c.into_element(),
            Shape::Text(t) => t.into_element(),
            Shape::Path(p) => p.into_element(),
        }
    }
}

pub trait IntoShape { fn into_shape(self) -> Shape; }

impl IntoShape for Rect { fn into_shape(self) -> Shape { Shape::Rect(self) } }
impl IntoShape for Circle { fn into_shape(self) -> Shape { Shape::Circle(self) } }
impl IntoShape for Text { fn into_shape(self) -> Shape { Shape::Text(self) } }
impl IntoShape for Path { fn into_shape(self) -> Shape { Shape::Path(self) } }

pub struct Svg {
    doc: Element,
}

impl IntoElement for Svg {
    fn into_element(self) -> Element { self.doc }
}

use std::fmt;

impl fmt::Display for Svg {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        self.doc.fmt(w)
    }
}

impl Svg {
    /// Create an SVG doc of given `width` and `height`.
    pub fn new(width: u32, height: u32) -> Svg {
        let mut doc = Element::new("svg");
        doc.insert_attribute("version", "1.1");
        doc.insert_attribute("baseProfile", "full");
        doc.insert_attribute("xmlns", "http://www.w3.org/2000/svg");
        doc.insert_attribute("width", width.to_string());
        doc.insert_attribute("height", height.to_string());
        Svg { doc: doc }
    }

    /// Add (or overwrite) an attribute on the SVG document.
    pub fn insert_attribute<S1, S2>(&mut self, attr: S1, val: S2)
        where S1: Into<String>, S2: Into<String>
    {
        self.doc.attributes.insert(attr.into(), val.into());
    }

    /// Returns the version of the SVG document. Documents start by default at version 1.1.
    pub fn version(&self) -> &str { &self.doc.attributes["version"] }

    /// Returns the element children of the SVG document.
    pub fn children(&self) -> &[Element] { &self.doc.children }

    /// Returns the width of the SVG document.
    pub fn width(&self) -> u32 { self.doc.attributes["width"].parse::<u32>().unwrap() }

    /// Returns the height of the SVG document.
    pub fn height(&self) -> u32 { self.doc.attributes["height"].parse::<u32>().unwrap() }

    /// Adds the given shape as a child element.
    pub fn add_child_shape<S:IntoShape>(&mut self, s: S) {
        self.doc.children.push(s.into_shape().into_element());
    }

    /// Adds the given named object as an (unrendered but referencable) definition.
    pub fn add_def<Def:Identified>(&mut self, def: Def) {
        let def = def.into_element();
        if let Some(defs) = self.doc.find_child_mut(|e| e.name == "defs") {
            defs.children.push(def);
            return;
        }
        self.doc.children.push(Element::new("defs"));
        let defs = self.doc.children.last_mut().unwrap();
        defs.children.push(def);
    }
}

pub struct Pattern {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub content: Vec<Shape>,
}

impl IntoElement for Pattern {
    fn into_element(self) -> Element {
        let mut e = Element::new("pattern");
        e.insert_attribute("id", self.id);
        e.insert_attribute("width", self.width.to_string());
        e.insert_attribute("height", self.height.to_string());
        e.insert_attribute("patternUnits", "userSpaceOnUse");
        for shape in self.content {
            e.children.push(shape.into_element());
        }
        e
    }
}

impl Identified for Pattern {
    fn id(&self) -> &str { &self.id }
}

/// `Identified` marks an XML element-structure that is known to carry
/// an `id` attribute, and thus is suitable for e.g. putting into a
/// `defs` block.
pub trait Identified: IntoElement {
    fn id(&self) -> &str;
}



#[cfg(test)]
mod tests {
    use super::{Svg, Fill, Color, Circle, Rect, Text, Dim};
    use super::{text};

    #[test]
    fn it_works() {
        let s = Svg::new(300, 200);
        assert_eq!(s.version(), "1.1");
        assert_eq!(s.width(), 300);
        assert_eq!(s.height(), 200);
    }

    #[test]
    fn dmo_simple_example() {
        let mut s = Svg::new(300, 200);
        s.add_child_shape(Rect { x: Dim::U(0,0),
                                 y: Dim::U(0,0),
                                 width: Dim::Pc(100,0),
                                 height: Dim::Pc(100,0),
                                 fill: Fill::Color(Color::Red),
                                 stroke: None,
                                 rounded: None,
        });
        s.add_child_shape(Circle { cx: Dim::U(150,0),
                                   cy: Dim::U(100,0),
                                   r: Dim::U(80,0),
                                   fill: Color::Green });
        s.add_child_shape(Text { x: Dim::U(150,0),
                                 y: Dim::U(125,0),
                                 font_size: Dim::U(60,0),
                                 text_anchor: text::TextAnchor::Middle,
                                 fill: Color::White,
                                 content: "SVG".to_string() });
    }
}
```
