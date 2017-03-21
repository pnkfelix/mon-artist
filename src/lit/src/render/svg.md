A drawing is essentially a set of paths on a backdrop.

(Note that we should not need to look at the `grid` itself;
 that is closer to the raw data we had input, while here we
 want to work at a higher, more abstract level.)

```rust
use directions::Direction;
use super::Scene;
use path::{self, Closed, Path};
use grid::{Pt};

use format;
use text;
```

The `RenderS` impl renders into an instance of the `Svg` structure
defined at the top-level.

```rust
use svg::{self, Color, Dim, Fill, Svg, Shape};

use super::{Render, RenderS};

use std::cmp;
```

At first I had thought `SvgRender` did not need
to carry any state; I thought all of the needed information
would come from the `Scene` alone.

```rust
pub struct SvgRender {
```

Then I remembered an important detail about rendering a Scene:
the width and height of an SVG document are measured
using the same metric unit, while the width and height
of a scene can use distinct units.

Therefore we need to know the scaling factor: how does
a width or height in the scene translate to the width/height
in the output SVG.

```rust
    // An output x-coordinate is an input times `x_scale`.
    pub x_scale: u32,
    // An output y-coordinate is an input times `y_scale`.
    pub y_scale: u32,

    pub font_family: String,
    pub font_size: u32,
```

In addition, we may well want to customize other aspects
of the rendering; in particular, I like to have a gray
grid pattern in the background indicating where the
characters would ideally lie if that space were filled
with text (which, for an ASCII-art based picture like
this, should be readily derived from the above values
for `x_scale` and `y_scale`.

```rust
    pub show_gridlines: bool,
```

A late addition: I think I initially wrote this code before
the format::Table even existed, and I managed to overlook
the need to pass it through here to the `PathRender`.
```rust
    pub format_table: format::Table,
```

While the automatic rendering of rectangle-like closed paths
into `<rect>` is a nice feature, it is a
deviation from the core model that I want to describe
(where nearly all of the interesting stuff is encoded in the format
table).
Therefore, allow a client to remove it so we can pretend it
does not exist.
```rust
    pub infer_rect_elements: bool,
```

And since I'm adding state anyway, I might as well provide
a place to include additional contextual information.
```rust
    pub name: String,
}

fn default<D: Default>() -> D { Default::default() }

impl RenderS for SvgRender {
    type Out = Svg;
    fn render_s(&self, scene: &Scene) -> Svg {
        render_svg(self, scene, &self.format_table)
    }
}

impl Render for SvgRender {
    fn render(&self, scene: &Scene) -> String {
        format!("{}", render_svg(self, scene, &self.format_table))
    }
}

fn render_svg(sr: &SvgRender, scene: &Scene, format_table: &format::Table) -> Svg {
    let mut svg = Svg::new(scene.width() * sr.x_scale,
                           scene.height() * sr.y_scale);
    if sr.show_gridlines {
        render_gridlines(&mut svg, sr);
    }

    debug!("rendering paths: {:?}", scene.paths());
    for path in scene.paths() {
        render_path(&mut svg, sr, path, format_table.clone());
    }

    debug!("rendering texts: {:?}", scene.texts());
    for text in scene.texts() {
        render_text(&mut svg, sr, text);
    }

    svg
}

fn render_gridlines(svg: &mut Svg, sr: &SvgRender) {
    let grid_cell = svg::Pattern {
        id: "grid_cell".to_string(),
        width: sr.x_scale,
        height: sr.y_scale,
        content: vec![Shape::Rect(svg::Rect {
            x: default(),
            y: default(),
            width: Dim::U(sr.x_scale,0),
            height: Dim::U(sr.y_scale,0),
            fill: Fill::None,
            stroke: Some((Fill::Color(Color::Gray), Dim::U(0,25))),
            rounded: None,
            id: None,
            attrs: vec![],
        })],
    };
    let grid = svg::Pattern {
        id: "grid".to_string(),
        width: grid_cell.width * 10,
        height: grid_cell.height * 10,
        content: vec![Shape::Rect(svg::Rect {
            x: default(),
            y: default(),
            width: Dim::U(grid_cell.width * 10,0),
            height: Dim::U(grid_cell.height * 10,0),
            fill: Fill::Pattern { def_id: "grid_cell".to_string(), },
            stroke: Some((Fill::Color(Color::DarkGray), Dim::U(0,5))),
            rounded: None,
            id: None,
            attrs: vec![],
        })],
    };
    svg.add_def(grid_cell);
    svg.add_def(grid);
    svg.add_child_shape(svg::Rect {
        x: Dim::U(0,0),
        y: Dim::U(0,0),
        width: Dim::Pc(100,0),
        height: Dim::Pc(100,0),
        fill: Fill::Pattern { def_id: "grid".to_string() },
        stroke: None,
        rounded: None,
        id: None,
        attrs: vec![],
    });
}
```

I want a simple mental model for what the guarantees are about where a
path will travel.

That is, I want the user to have a reasonable chance of predicting
what pixels are likely to be covered by a path when it is rendered,
and what the path overall is going to look like.

At one point I considered having the goal be "for each pt on a path,
the rendering always travels through the center of every grid cell
corresponding to that pt." But that is actually not the right thing;

 * For example, when a path curves, the easiest way for me to
   accomplish that is via a bezier curve.

 * The first question that raises is: what control point should I use
   to ensure that a curve goes through the center of a grid?

 * Yet that is not even the right question. The right question is:
   Would the curve even *look good* if it were to go through the
   center of the grid cell?

 * After some thought on the matter, I concluded that the right thing
   here is not to enforce a rule that paths go through the center of the grid
   cells, but instead to enforce a rule that for any neighboring points (pt, pt')
   on a path, the rendering will go through the pixel on the border between
   pt and pt' that is nearest to their centers.

 * That pixel corresponds to one of the eight compass points for each
   of the grid cells for those pts, so this is actually a pretty
   intuitive property to enforce.


```rust
fn render_path(svg: &mut Svg,
               sr: &SvgRender,
               path: &Path,
               format_table: format::Table) {
    #![allow(unused_parens)]
```

We special case rendering rectangles (at least for now)
to detect when we can use the `<rect>` element rather
than a general `<path>` element, since I assume that
all clients would prefer the former when possible.

```rust
    debug!("rendering path: {:?} rectangular: {:?}", path, path.is_rectangular());
    if sr.infer_rect_elements {
        if let Some(corners) = path.is_rectangular() {
            let opt_id = path.id.as_ref().map(|&(_, ref s)|s.clone());
            match render_rectangle(svg, sr, opt_id, &path.attrs, corners) {
                Ok(_) => return,
                Err(_) => {} // fall through to general case below.
            }
        }
    }
```

The general pattern of all of these things is to look at
each triplet (or, at the edges, pairs) of adjacent steps
to figure out how to render the middle (or edge) step.

```rust
    let len = path.steps.len();
    let steps = &path.steps[..];
    assert!(len >= 2);

    // assume undashed until proven otherwise.
    let mut pr = PathRender { sr: sr, last: Pt(0,0),
                              dashed: false,
                              cmd: String::new(),
                              format_table: format_table,
                              attrs: Vec::new(),
    };

    match path.closed {
        Closed::Closed => {
            pr.render_first_step_in_loop(*steps.last().unwrap(),
                                         steps[0],
                                         steps[1]);
        }
        Closed::Open => {
            pr.render_first_step(steps[0], steps[1]);
        }
    }

    for i in 2..len {
        pr.render_middle_step(steps[i-2], steps[i-1], steps[i]);
    }

    pr.render_last_step(steps[len-2], steps[len-1], path.closed);

    if let Some((_, ref id)) = path.id {
        pr.attrs.push(("id".to_string(), id.clone()));
    }

    if let Some(ref attrs) = path.attrs {
        for &(ref k, ref v) in attrs {
            pr.attrs.push((k.to_string(), v.to_string()));
        }
    }

    debug!("Path {:?} yields cmd: {:?}", path, pr.cmd);
    svg.add_child_shape(pr.into_shape())
}

fn render_text(svg: &mut Svg, sr: &SvgRender, text: &text::Text) {
    use svg::text as svg_text;
    let place = interpret_place(sr, "W", None, text.pt, None);
    debug!("rendering text: {:?} starting at place: {:?}", text, place);
    let rendered = svg_text::Text {
        x: place.0,
        y: place.1,
        font_family: sr.font_family.clone(),
        font_size: svg::Dim::U(sr.font_size,0),
        text_anchor: svg_text::TextAnchor::Start,
        fill: Color::Black,
        content: text.content.clone(),
        id: text.id.clone().map(|(_, s)|s),
        attrs: text.attrs.clone().unwrap_or(vec![]),
    };
    debug!("rendered text: {:?} to {:?}", text, rendered);
    svg.add_child_shape(rendered);
}

type Step = (Pt, char);
struct PathRender<'a> {
    sr: &'a SvgRender,
    #[allow(dead_code)]
    last: Pt,
    dashed: bool,
    cmd: String,
    format_table: format::Table,
    attrs: Vec<(String, String)>,
}

impl<'a> PathRender<'a> {
    fn render_first_step_in_loop(&mut self, last: Step, curr: Step, next: Step) {
        let (c, attrs) = render_loop_start(self, last, curr, next);
        self.cmd.push_str(&c);
        for attr in attrs { if !self.attrs.contains(&attr) { self.attrs.push(attr) } }
    }
    fn render_first_step(&mut self, curr: Step, next: Step) {
        let (c, attrs) = render_step(self, None, curr, Some(next));
        self.cmd.push_str(&c);
        for attr in attrs { if !self.attrs.contains(&attr) { self.attrs.push(attr) } }
    }
    fn render_middle_step(&mut self, prev: Step, curr: Step, next: Step) {
        let (c, attrs) = render_step(self, Some(prev), curr, Some(next));
        self.cmd.push_str(&c);
        for attr in attrs { if !self.attrs.contains(&attr) { self.attrs.push(attr) } }
    }
    fn render_last_step(&mut self, prev: Step, curr: Step, cd: path::Closed) {
        let (c, attrs) = render_step(self, Some(prev), curr, None);
        self.cmd.push_str(&c);
        if cd == path::Closed::Closed { self.cmd.push_str(" Z"); }
        for attr in attrs { if !self.attrs.contains(&attr) { self.attrs.push(attr) } }
    }
    fn into_shape(self) -> svg::Path {
        let mut attrs = self.attrs.clone();
        if attrs.iter().find(|a|a.0 == "fill").is_none() {
            attrs.push(("fill".to_string(), "none".to_string()));
        }
        if attrs.iter().find(|a|a.0 == "stroke").is_none() {
            attrs.push(("stroke".to_string(), "green".to_string()));
        }
        if attrs.iter().find(|a|a.0 == "stroke-width").is_none() {
            attrs.push(("stroke-width".to_string(), "2".to_string()));
        }
        if self.dashed && attrs.iter().find(|a|a.0 == "stroke-dasharray").is_none() {
            attrs.push(("stroke-dasharray".to_string(), "1,1".to_string()));
        }
        svg::Path { d: self.cmd, attrs: attrs }
    }

    fn substitute_placeholders(&self,
                               template: &str,
                               incoming: Option<Direction>,
                               curr: Step,
                               outgoing: Option<Direction>) -> String
    {
        let mut s = String::new();
        let mut rest = &*template;
        loop {
            let (place, j) = match rest.find("{") {
                None => { s.push_str(rest); return s; }
                Some(i) => {
                    s.push_str(&rest[..i]);
                    let place = &rest[i..];
                    match place.find("}") {
                        None => { panic!("open { without matching close }"); }
                        Some(j) => (place, j)
                    }
                }
            };
            rest = &place[(j+1)..]; // j+1: skip the "}"
            let place = &place[1..j]; // 1: skip the "{"
            let (value_x, value_y) =
                interpret_place(self.sr, &place, incoming, curr.0, outgoing);
            s.push_str(&format!("{},{}", value_x.to_string(), value_y.to_string()));
        }
    }
}

struct CompassPoints<T> {
    n: T, s: T, e: T, w: T, ne: T, se: T, sw: T, nw: T
}
```

The template string in a path-search-and-rendering rule is built from
a domain-specific language for describing how to render a given
character.

It uses SVG path data syntax, with special placeholder components for
describing values that need to be plugged in.

The format of the plugged in values is either:

* A primitive point, or
* A point along the line connecting any of the two of the above nine points.
  (note: this still remains to be implemented).

where a primitive point is either

* The center of the current grid cell, or
* One of the eight compass oriented extremities on the edge around
  the current grid cell.

(At some point I may add support for other primitive points, such
as points on the predecessor or successor grid cell. But for now
the intention is to only make it easy to describe paths relative
to the current grid cell.)

The syntax for specifying a placeholder value is bracket delimited.

For the nine primitive point cases (i.e. center or edge), one may write
one of the following as appropriate:

`{C}`, `{N}`, `{NE}`, `{E}`, `{SE}`, `{S}`, `{SW}, `{W}`, `{NW}`,

In addition, one can refer to an edge defined in terms of the
incoming (`I`) or outgoing (`O`) node using one of the following:

`{I}`, `{O}`, `{RI}`, `{RO}`

`{I}` is the edge from which we came; likewise `{O}` is the outgoing
neighbor. `{RI}` and `{RO}` are the *reflections* of those points.

* For example, if the incoming neighbor is to the northeast, then `{I}`
  is the same as `{NE}` and `{RI}` is the same as `{SW}`.

(Unimplemented:)
For a point along a line, one writes a decimal number in the range
[0,1] (followed by optional non-linebreak whitespace), followed by
two of the above base cases, delimited by a `-` mark (and again one
is allowed to include non-linebreak whitespace before and after the
`-`).

* For example, the point that is 3/10 of the way along the path from
  the center to the north-east corner could be written `{.3 C-NE}`.

The substituted value for the placeholder will be the absolute x,y
coordinates for the described point. Note that this means that one
should usually use the capital letter commands, which take absolute
coordinates as inputs, in tandem with placeholders.

TODO: it might be a good idea to add lower-case analogous placeholders
that are then just ways to compute based on the width or height of the
grid cell. E.g. `{n}` would be replaced with `0,-6` if the cell
height is 12.


```rust
#[allow(warnings)]
fn interpret_place(sr: &SvgRender,
                   place: &str,
                   incoming: Option<Direction>,
                   curr: Pt,
                   outgoing: Option<Direction>) -> (Dim, Dim) {
    use directions::Direction::*;
    // x- and y-lines for compass points if all grid cells were 1x1 unit.
    let (ex, cx, wx) = {
        let x = curr.col() as u32;
        let ex = Dim::U(x,0);
        let cx = ex.sub_half();
        let wx = cx.sub_half();
        (ex, cx, wx)
    };
    let (sy, cy, ny) = {
        let y = curr.row() as u32;
        let sy = Dim::U(y,0);
        let cy = sy.sub_half();
        let ny = cy.sub_half();
        (sy, cy, ny)
    };

    // update lines of compass points according to scale provided by `sr`
    let (ex, cx, wx) = (ex * sr.x_scale,
                        cx * sr.x_scale,
                        wx * sr.x_scale);
    let (sy, cy, ny) = (sy * sr.y_scale,
                        cy * sr.y_scale,
                        ny * sr.y_scale);

    // coordinates for actual compass points
    let c = (cx, cy);

    let grid = CompassPoints {
        n: (cx, ny),
        s: (cx, sy),
        e: (ex, cy),
        w: (wx, cy),
        ne: (ex, ny),
        se: (ex, sy),
        nw: (wx, ny),
        sw: (wx, sy),
    };

    // If we have a circle, then it is largest possible given bounds
    // of each cell; its radius is half of the min of the two scales.
    let radius = cmp::min(sr.x_scale, sr.y_scale) as f64 / 2.0;

    // the diagonal corners, when projected onto the aforementioned
    // circle, fall on points on a 45-45-90 triangle whose hypotenuse
    // is the above radius, and thus the base and height of those
    // 45-45-90 triangles is radius / sqrt(2).
    let diagonal_on_circle_side = radius / (2.0f64).sqrt();

    let circle = {
        let d = diagonal_on_circle_side;
        CompassPoints {
            n: (cx, cy.subf(radius)),
            s: (cx, cy.addf(radius)),
            e: (cx.addf(radius), cy),
            w: (cx.subf(radius), cy),
            ne: (cx.addf(d), cy.subf(d)),
            se: (cx.addf(d), cy.addf(d)),
            sw: (cx.subf(d), cy.addf(d)),
            nw: (cx.subf(d), cy.subf(d)),
        }
    };

    // FIXME: these cover the edge of the grid cell itself, but
    // another case that I want is the compass points along the
    // circumference of the circle *circumscribed* by the grid cell.
    //
    // With that in hand, I could have consistent handling of things
    // like:
    //        /  \/
    // -o-  -o   o
    //
    // where I want the circle to be rendered with the same dimensions
    // in each case but I also want the lines to go right up to the
    // edge of the circle.
    //
    // Perhaps the syntax "DIR/o" could be used here? Or "(DIR)"? Hmm?
    // (The reasoning behind "DIR/o" is that its dividing the original
    // DIR by some amount >= 1, but the actual amount depends on the
    // direction (and also the actual grid dimensions too, of course).
    //
    // But if I add "DIR/o", I might as well work on adding the other
    // forms like "DIR/2" etc, right?

    let (numer, div) = match place.find("/") {
        None => (place, None),
        Some(i) => (&place[0..i], Some(&place[(i+1)..])),
    };

    let CompassPoints { n, s, e, w, ne, se, sw, nw } = match div {
        None => grid,
        Some("o") => circle,
        Some(divisor) => panic!("unrecognized divisor: {}", divisor),
    };

    match numer {
        "C" => c, "N" => n, "S" => s, "E" => e, "W" => w,
        "NE" => ne, "SE" => se, "NW" => nw, "SW" => sw,
        "I" => match incoming.unwrap_or_else(||panic!("cannot use `I` when none incoming")) {
            N => s, NE => sw, E => w, SE => nw, S => n, SW => ne, W => e, NW => se,
        },
        "RI" => match incoming.unwrap_or_else(||panic!("cannot use `RI` when none incoming")) {
            N => n, NE => ne, E => e, SE => se, S => s, SW => sw, W => w, NW => nw,
        },
        "O" => match outgoing.unwrap_or_else(||panic!("cannot use `O` when none outgoing")) {
            N => n, NE => ne, E => e, SE => se, S => s, SW => sw, W => w, NW => nw,
        },
        "RO" => match outgoing.unwrap_or_else(||panic!("cannot use `RO` when none outgoing")) {
            N => s, NE => sw, E => w, SE => nw, S => n, SW => ne, W => e, NW => se,
        },
        // FIXME: need to add support for points along lines as well.
        _ => panic!("unrecognized place: {}", place),
    }
}

fn to_incoming(prev: Option<Step>, curr: Step) -> Option<(char, Direction)> {
    prev.map(|p| (p.1, p.0.towards(curr.0)))
}

fn to_outgoing(curr: Step, next: Option<Step>) -> Option<(Direction, char)> {
    next.map(|n| (curr.0.towards(n.0), n.1))
}

fn render_loop_start(pr: &mut PathRender, prev: Step, curr: Step, next: Step)
                     -> (String, Vec<(String, String)>) {
    use directions::Direction;

    let sr = pr.sr;
    let t = &pr.format_table;

    let incoming: (char, Direction) = (prev.1, prev.0.towards(curr.0));
    let outgoing: (Direction, char) = (curr.0.towards(next.0), next.1);

    if let Some((template, attributes)) = t.find_loop(&|_| {}, incoming, curr.1, outgoing) {
        return (pr.substitute_placeholders(template,
                                           Some(incoming.1),
                                           curr,
                                           Some(outgoing.0)),
                attributes.iter().cloned().collect());
    } else {
        panic!("no command template found for prev: {:?} curr: {:?} next: {:?} name: {}",
               prev, curr, next, sr.name);
    }
}

fn render_step(pr: &mut PathRender, prev: Option<Step>, curr: Step, next: Option<Step>)
               -> (String, Vec<(String, String)>) {
    use directions::Direction;

    let sr = pr.sr;
    let t = &pr.format_table;

    let incoming: Option<(char, Direction)> = to_incoming(prev, curr);
    let outgoing: Option<(Direction, char)> = to_outgoing(curr, next);

    if let Some((template, attributes)) = t.find(&|_|{}, incoming, curr.1, outgoing) {
        return (pr.substitute_placeholders(template,
                                           incoming.map(|t|t.1),
                                           curr,
                                           outgoing.map(|t|t.0)),
                attributes.iter().cloned().collect());
    } else {
        panic!("no command template found for prev: {:?} curr: {:?} next: {:?} name: {}",
               prev, curr, next, sr.name);
    }
}

fn render_rectangle(svg: &mut Svg,
                    sr: &SvgRender,
                    id: Option<String>,
                    attrs: &Option<Vec<(String, String)>>,
                    corners: [(Pt, char); 4]) -> Result<(), ()> {
    if let Some((ul, ur, bl, rounded)) = match corners {
        // simple rectangle with sharp corners
        [(ul, '+'), (ur, '+'), (bl, '+'), (_, '+')] =>
            Some((ul, ur, bl, false)),
        // rounded-rectangle
        [(ul, '.'), (ur, '.'), (bl, '\''), (_, '\'')] =>
            Some((ul, ur, bl, true)),
        _ => None
    } {
        let x = ul.col() as u32;
        let x = Dim::U(x,0);
        let x = x.sub_half();
        let x = x * sr.x_scale;
        let y = ul.row() as u32;
            let y = Dim::U(y,0);
        let y = y.sub_half();
        let y = y * sr.y_scale;
        let width = (ur.col() - ul.col()) as u32;
        let width = Dim::U(width,0);
        let width = width * sr.x_scale;
        let height = (bl.row() - ul.row()) as u32;
        let height = Dim::U(height,0);
        let height = height * sr.y_scale;
        let mut new_attrs = vec![];
        if let Some(ref attrs) = *attrs {
            for &(ref k, ref v) in attrs {
                new_attrs.push((k.clone(), v.clone()));
            }
        }
        let rect = svg::Rect {
            x: x,
            y: y,
            width: width,
            height: height,
            fill: Fill::None,
            stroke: Some((Fill::Color(Color::Red), Dim::U(2,0))),
            rounded: if rounded {
                Some((Dim::U(0,5) * sr.x_scale, Dim::U(0,5) * sr.y_scale))
            } else {
                None
            },
            id: id,
            attrs: new_attrs,
        };
        svg.add_child_shape(rect);
        return Ok(());
    }

    // something else; handle it in general case elsewhere
    return Err(());
}
```
