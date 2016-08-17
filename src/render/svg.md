A drawing is essentially a set of paths on a backdrop.

(Note that we should not need to look at the `grid` itself;
 that is closer to the raw data we had input, while here we
 want to work at a higher, more abstract level.)

```rust
use directions::Direction;
use Scene;
use path::{self, Path};
use grid::{Pt};

use format;
```

The `RenderS` impl renders into an instance of the `Svg` structure
defined at the top-level.

```rust
use svg::{self, Color, Dim, Fill, Svg, Shape};

use super::{Render, RenderS};
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

And since I'm adding state anyway, I might as well provide
a place to include additional contextual information.
```rust
    pub(crate) name: String,
}

fn default<D: Default>() -> D { Default::default() }

impl RenderS for SvgRender {
    type Out = Svg;
    fn render_s(&self, scene: &Scene) -> Svg {
        render_svg(self, scene)
    }
}

impl Render for SvgRender {
    fn render(&self, scene: &Scene) -> String {
        format!("{}", render_svg(self, scene))
    }
}

fn render_svg(sr: &SvgRender, scene: &Scene) -> Svg {
    let mut svg = Svg::new(scene.width() * sr.x_scale,
                           scene.height() * sr.y_scale);
    if sr.show_gridlines {
        render_gridlines(&mut svg, sr);
    }

    debug!("rendering paths: {:?}", scene.paths());
    for path in scene.paths() {
        render_path(&mut svg, sr, path);
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
#[allow(dead_code)]
fn grid_middle(sr: &SvgRender, pt: Pt) -> (Dim, Dim) {
    let x = Dim::U(pt.col() as u32,0).sub_half() * sr.x_scale;
    let y = Dim::U(pt.row() as u32,0).sub_half() * sr.y_scale;
    (x, y)
}

#[allow(dead_code)]
fn is_line(c: Option<char>) -> bool {
    match c {
        Some(c) => match c {
            '|' | ':' | '/' | '-' | '=' | '\\' => true,
            // FIXME none of these belong here
            '<' | '>' | 'V' | '^' => true,
            _ => false,
        },
        None => false,
    }
}

#[allow(dead_code)]
fn is_curve(c: Option<char>) -> bool {
    match c {
        Some(c) => match c { '.' | '\'' => true, _ => false, },
        None => false,
    }
}

#[allow(dead_code)]
fn midway(sr: &SvgRender, p1: Pt, p2: Pt) -> (Dim, Dim) {
    let x = Dim::U(p1.col() as u32 + p2.col() as u32,0).div_2() * sr.x_scale;
    let y = Dim::U(p1.row() as u32 + p2.row() as u32,0).div_2() * sr.y_scale;
    (x, y)
}

fn render_path(svg: &mut Svg, sr: &SvgRender, path: &Path) {
    #![allow(unused_parens)]
```

We special case rendering rectangles (at least for now)
to detect when we can use the `<rect>` element rather
than a general `<path>` element, since I assume that
all clients would prefer the former when possible.

```rust
    debug!("rendering path: {:?} rectangular: {:?}", path, path.is_rectangular());
    if let Some(corners) = path.is_rectangular() {
        match render_rectangle(svg, sr, corners) {
            Ok(_) => return,
            Err(_) => {} // fall through to general case below.
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
                              format_table: Default::default(),
    };

    pr.render_first_step(steps[0], steps[1]);

    for i in 2..len {
        pr.render_middle_step(steps[i-2], steps[i-1], steps[i]);
    }

    pr.render_last_step(steps[len-2], steps[len-1], path.closed);

    debug!("Path {:?} yields cmd: {:?}", path, pr.cmd);
    svg.add_child_shape(pr.into_shape())
}

type Step = (Pt, char);
struct PathRender<'a> {
    sr: &'a SvgRender,
    #[allow(dead_code)]
    last: Pt,
    dashed: bool,
    cmd: String,
    #[allow(dead_code)]
    format_table: format::Table,
}

impl<'a> PathRender<'a> {
    fn render_first_step(&mut self, curr: Step, next: Step) {
        let c = render_step(self, None, curr, Some(next));
        self.cmd.push_str(&c);
    }
    fn render_middle_step(&mut self, prev: Step, curr: Step, next: Step) {
        let c = render_step(self, Some(prev), curr, Some(next));
        self.cmd.push_str(&c);
    }
    fn render_last_step(&mut self, prev: Step, curr: Step, cd: path::Closed) {
        let c = render_step(self, Some(prev), curr, None);
        self.cmd.push_str(&c);
        if cd == path::Closed::Closed { self.cmd.push_str(" Z"); }
    }
    fn into_shape(self) -> svg::Path {
        let mut attrs = vec![("fill".to_string(),"none".to_string()),
                             ("stroke".to_string(),"green".to_string())];
        if self.dashed {
            attrs.push(("stroke-dasharray".to_string(), "1,1".to_string()));
        }
        svg::Path { d: self.cmd, attrs: attrs }
    }

    fn substitute_placeholders(&self,
                               template: &str,
                               curr: Step) -> String
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
            let (value_x, value_y) = interpret_place(self.sr, &place, curr.0);
            s.push_str(&format!("{},{}", value_x.to_string(), value_y.to_string()));
        }
    }
}

#[allow(warnings)]
fn interpret_place(sr: &SvgRender, place: &str, curr: Pt) -> (Dim, Dim) {
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
    let n = (cx, ny);
    let s = (cx, sy);
    let e = (ex, cy);
    let w = (wx, cy);
    let ne = (ex, ny);
    let se = (ex, sy);
    let nw = (wx, ny);
    let sw = (wx, sy);

    match place {
        "C" => c, "N" => n, "S" => s, "E" => e, "W" => w,
        "NE" => ne, "SE" => se, "NW" => nw, "SW" => sw,
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

fn render_step(pr: &mut PathRender, prev: Option<Step>, curr: Step, next: Option<Step>) -> String {
    use directions::Direction;

    let sr = pr.sr;
    let t = &pr.format_table;

    let incoming: Option<(char, Direction)> = to_incoming(prev, curr);
    let outgoing: Option<(Direction, char)> = to_outgoing(curr, next);

    if let Some(s) = t.find(incoming, curr.1, outgoing) {
        return pr.substitute_placeholders(s, curr);
    } else {
        panic!("no command template found for prev: {:?} curr: {:?} next: {:?} name: {}",
               prev, curr, next, sr.name);
    }

/*
    let mut cmd = &mut pr.cmd;
    // Where are we starting?
    // If we are continuing from a previous segment, then just the value stored in `pr`.

    let last;
    if prev.is_some() {
        last = pr.last;
    } else {
        // First we have to figure out what the first step of the path is.
        let ((pt0, c0), (pt1, c1)) = (curr, next.unwrap());
        match (c0, c1) {
            ('-', '-') => unimplemented!(),
            _ => unimplemented!(),
        }

        // Normally the center of the grid cell will do, but there are exceptions.
        let start = {
            let x = pt0.col() as u32;
            let x = match (c0, c1) {
                ('-', _) => {
                    if pt1.col() > pt0.col() {
                        Dim::U(x-1,0)
                    } else {
                        Dim::U(x,0)
                    }
                }
                _ => {
                    Dim::U(x,0).sub_half()
                }
            };

            let y = pt0.row() as u32;
            let y = match (c0, c1) {
                // `.` and `'` are special cased on next elem vertical
                ('.', '|') | ('\'', '|') => {
                    // in which case we start from the the grid middle.
                    Dim::U(y,0).sub_half()
                }
                // otherwise, `.` starts from middle of bottom edge
                ('.', _) => {
                    Dim::U(y,0)
                }
                // and `'` starts from middle of top edge
                ('\'', _) => {
                    Dim::U(y-1, 0)
                }
                ('|', _) => {
                    if pt1.row() > pt0.row() {
                        Dim::U(y-1,0)
                    } else {
                        Dim::U(y,0)
                    }
                }
                // for now all other cases also start from grid middle
                _ => {
                    Dim::U(y,0).sub_half()
                }
            };
            (x * sr.x_scale, y * sr.y_scale)
        };
        last = pt0;
        cmd.push_str(&format!("M {} {}", &start.0.to_string(), &start.1.to_string()));
    }

    let (pt, c) = curr;
    let tgt = match c {
        '\'' | '.' if next.is_some() => {
            let (next, _) = next.unwrap();
            midway(sr, pt, next)
        }
        _ => grid_middle(sr, pt),
    };
    let prev_c = prev.map(|(_,c)|c);
    if (prev_c == Some('+') || is_line(prev_c)) && (is_line(Some(c)) || c == '+') {
        cmd.push_str(&format!(" L {},{}", &tgt.0.to_string(), &tgt.1.to_string()));
    } else if (is_curve(prev_c) && is_line(Some(c))) || (is_line(prev_c) && is_curve(Some(c))) {
        let ctrl = grid_middle(sr, last);
        cmd.push_str(&format!(" Q {},{} {},{}", ctrl.0.to_string(), ctrl.1.to_string(), tgt.0.to_string(), tgt.1.to_string()));
    } else {
        panic!("unimplemented: last: {:?} steps: {:?} c: {}", last, (prev, curr, next), c);
    }
    if c == ':' || c == '=' {
        pr.dashed = true;
    }

    pr.last = pt;

    unimplemented!()
*/
}

fn render_rectangle(svg: &mut Svg,
                    sr: &SvgRender,
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
        let rect = svg::Rect {
            x: x,
            y: y,
            width: width,
            height: height,
            fill: Fill::None,
            stroke: Some((Fill::Color(Color::Red), Dim::U(4,0))),
            rounded: if rounded {
                Some((Dim::U(0,5) * sr.x_scale, Dim::U(0,5) * sr.y_scale))
            } else {
                None
            },
        };
        svg.add_child_shape(rect);
        return Ok(());
    }

    // something else; handle it in general case elsewhere
    return Err(());
}
```
