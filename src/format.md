```rust
use directions::{self, Direction, ToDirections};

#[derive(Clone, Debug)]
pub enum Match {
    One(char),

    Chars(Vec<char>),

    // matches any non-blank character
    Any,
}

impl Match {
    pub fn matches(&self, c: char) -> bool {
        match *self {
            Match::One(m) => m == c,
            Match::Chars(ref v) => v.contains(&c),
            Match::Any => !c.is_whitespace(),
        }
    }
}

pub trait IntoMatch { fn into_match(self) -> Match; }

impl IntoMatch for Match { fn into_match(self) -> Match { self } }
impl IntoMatch for char { fn into_match(self) -> Match { Match::One(self) } }
impl IntoMatch for Vec<char> { fn into_match(self) -> Match { Match::Chars(self) } }
impl<'a> IntoMatch for &'a str {
    fn into_match(self) -> Match { Match::Chars(self.chars().collect()) }
}
impl IntoMatch for String {
    fn into_match(self) -> Match { (self[..]).into_match() }
}

#[derive(Clone, Debug)]
pub enum Neighbor<T> {
    /// no neighbor allowed (i.e. pattern for some end of the path).
    Blank,
    /// must match some non-blank neighbor
    Must(T),
    /// may match some non-blank neighbor, but also matches an end of the path.
    May(T),
}

/// Each Entry describes how to render a character along a path,
/// based on the context in which it appears.
#[derive(Clone, Debug)]
pub struct Entry {
    entry_text: &'static str,

    /// `loop_start` is true if this entry represents a starting point
    /// for a closed polygon, e.g. a corner `+` is one such character.
    ///
    /// FIXME: there are impossible states (like Blank
    /// incoming/outgoing + loop_start true).  would be better to
    /// revise representation, e.g. with an enum {
    /// Edge(in,curr,out,is_loop),
    pub(crate) loop_start: bool,

    /// `Blank` if the first step in path; otherwise, the set of
    /// previous characters matched by this entry and direction from
    /// the previous step into `curr`.
    incoming: Neighbor<(Match, Vec<Direction>)>,

    /// The set of current characters matched by this entry.
    curr: Match,

    /// `Blank` if the last step in path; otherwise, direction from
    /// `curr` into next step, and the set of characters for next step
    /// matched by this entry.
    outgoing: Neighbor<(Vec<Direction>, Match)>,

    /// The template to use when rendering `curr`.
    template: String,

    /// attribute(s) that should be present on element if this pattern
    /// is matched along the path.
    include_attributes: Vec<(String, String)>,

    /// If `instrumented` is true, then during rendering we will
    /// announce a message (i.e. invoke a callback) every time this
    /// entry is considered, including the `entry_text`, the actual
    /// inputs under consideration, and the returned result.
    pub(crate) instrumented: bool,
}

impl Entry {
    pub fn incoming(&self) -> Neighbor<(Match, Vec<Direction>)> {
        self.incoming.clone()
    }

    pub fn outgoing(&self) -> Neighbor<(Vec<Direction>, Match)> {
        self.outgoing.clone()
    }
}

pub(crate) type Announce<'a> = &'a Fn(String);

impl Entry {
    pub(crate) fn matches_curr(&self, _a: Announce, curr: char) -> bool {
        let ret = self.curr.matches(curr);
        // if self.instrumented {
        //     a(format!("matches_curr({:?}) on {} => {:?}",
        //               curr, self.entry_text, ret));
        // }
        ret
    }

    fn matches_incoming(&self, _a: Announce, incoming: Option<(char, Direction)>) -> bool {
        use self::Neighbor::{Blank, Must, May};
        let ret = match (&self.incoming, &incoming) {
            (&Blank, &Some(_)) | (&Must(..), &None) => false,
            (&Blank, &None) | (&May(..), &None) => true,
            (&Must((ref m, ref dirs)), &Some((c, d))) |
            (&May((ref m, ref dirs)), &Some((c, d))) =>
                if !dirs.contains(&d) {
                    false
                } else if !m.matches(c) {
                    false
                } else {
                    true
                },
        };
        // if self.instrumented {
        //     a(format!("matches_incoming({:?}) on {} => {:?}",
        //               incoming, self.entry_text, ret));
        // }
        ret
    }

    fn matches_outgoing(&self, _a: Announce, outgoing: Option<(Direction, char)>) -> bool {
        use self::Neighbor::{Blank, Must, May};
        let ret = match (&self.outgoing, &outgoing) {
            (&Blank, &Some(_)) | (&Must(..), &None) => false,
            (&Blank, &None) | (&May(..), &None) => true,
            (&May((ref dirs, ref m)), &Some((d, c))) |
            (&Must((ref dirs, ref m)), &Some((d, c))) =>
                if !dirs.contains(&d) {
                    false
                } else if !m.matches(c) {
                    false
                } else {
                    true
                },
        };
        // if self.instrumented {
        //     a(format!("matches_outgoing({:?}) on {} => {:?}",
        //               outgoing, self.entry_text, ret));
        // }
        ret
    }

    pub(crate) fn matches(&self,
                          a: Announce,
                          incoming: Option<(char, Direction)>,
                          curr: char,
                          outgoing: Option<(Direction, char)>) -> bool {
        let ret = if !self.matches_incoming(a, incoming) { false
        } else if !self.curr.matches(curr) {
            false
        } else if !self.matches_outgoing(a, outgoing) {
            false
        } else {
            true
        };
        if self.instrumented {
            a(format!("matches({:?}, {:?}, {:?}) on {} => {:?}",
                      incoming, curr, outgoing, self.entry_text, ret));
        }
        ret
    }

    pub(crate) fn matches_start(&self,
                                a: Announce,
                                curr: char,
                                outgoing: Option<(Direction, char)>) -> bool {
        use self::Neighbor::{Blank, Must, May};
        let ret = match &self.incoming {
            &Blank | &May(..) => {
                if !self.curr.matches(curr) {
                    false
                } else if !self.matches_outgoing(a, outgoing) {
                    false
                } else {
                    true
                }
            }
            &Must(..) => false,
        };
        if self.instrumented {
            a(format!("matches_start({:?}, {:?}) on {} => {:?}",
                      curr, outgoing, self.entry_text, ret));
        }
        ret
    }

    pub(crate) fn matches_end(&self,
                              a: Announce,
                              incoming: Option<(char, Direction)>,
                              curr: char) -> bool {
        use self::Neighbor::{Blank, Must, May};
        let ret = if !self.matches_incoming(a, incoming) {
            false
        } else if !self.curr.matches(curr) {
            false
        } else {
            match &self.outgoing {
                &Blank | &May(..) => true,
                &Must(..) => false,
            }
        };
        if self.instrumented {
            a(format!("matches_end({:?}, {:?}) on {} => {:?}",
                      incoming, curr, self.entry_text, ret));
        }
        ret
    }
}

impl Entry {
    pub(crate) fn corner_incoming(&self) -> (Match, Vec<Direction>) {
        match self.incoming {
            Neighbor::Blank => panic!("A loop_start cannot require blank neighbor"),
            Neighbor::May(ref t) | Neighbor::Must(ref t) => t.clone(),
        }
    }

    pub(crate) fn corner_outgoing(&self) -> (Vec<Direction>, Match) {
        match self.outgoing {
            Neighbor::Blank => panic!("A loop_start cannot require blank neighbor"),
            Neighbor::May(ref t) | Neighbor::Must(ref t) => t.clone(),
        }
    }
}

pub trait IntoAttributes { fn into_attributes(self) -> Vec<(String, String)>; }
impl IntoAttributes for () { fn into_attributes(self) -> Vec<(String, String)> { vec![] } }
impl IntoAttributes for [(&'static str, &'static str); 1] {
    fn into_attributes(self) -> Vec<(String, String)> {
        self.into_iter().map(|&(a,b)|(a.to_string(), b.to_string())).collect()
    }
}

pub trait IntoEntry { fn into_entry(self, text: &'static str) -> Entry; }

pub trait IntoCurr: IntoMatch { fn is_loop(&self) -> bool { false } }

/// Use `All` to match either the end of the path or any non-blank character.
pub struct All;

/// Use `May` to match either the end of the path or a particular match
pub struct May<C>(C);

/// Use `Loop` to match a corner for a closed polygon.
pub struct Loop<C>(C);


impl<C:IntoMatch> IntoMatch for Loop<C> {
    fn into_match(self) -> Match { self.0.into_match() }

}
impl<C:IntoMatch> IntoCurr for Loop<C> {
    fn is_loop(&self) -> bool { true }
}

impl IntoCurr for Match { }
impl IntoCurr for char { }
impl IntoCurr for Vec<char> { }
impl<'a> IntoCurr for &'a str { }
impl IntoCurr for String { }

impl<'a, C0, D0, C1, D1, C2> IntoEntry for (C0, D0, C1, D1, C2, &'a str) where
    C0: IntoMatch, D0: ToDirections, C1: IntoCurr, D1: ToDirections, C2: IntoMatch
{
    fn into_entry(self, text: &'static str) -> Entry {
        use self::Neighbor::{Must};
        Entry {
            instrumented: false,
            entry_text: text,
            loop_start: self.2.is_loop(),
            incoming: Must((self.0.into_match(), self.1.to_directions())),
            curr: self.2.into_match(),
            outgoing: Must((self.3.to_directions(), self.4.into_match())),
            template: self.5.to_string(),
            include_attributes: vec![],
        }
    }
}

impl<'a, C0, D0, C1, D1, C2, A> IntoEntry for (C0, D0, C1, D1, C2, &'a str, A) where
    C0: IntoMatch, D0: ToDirections, C1: IntoCurr, D1: ToDirections, C2: IntoMatch, A: IntoAttributes
{
    fn into_entry(self, text: &'static str) -> Entry {
        use self::Neighbor::{Must};
        Entry {
            instrumented: false,
            entry_text: text,
            loop_start: self.2.is_loop(),
            incoming: Must((self.0.into_match(), self.1.to_directions())),
            curr: self.2.into_match(),
            outgoing: Must((self.3.to_directions(), self.4.into_match())),
            template: self.5.to_string(),
            include_attributes: self.6.into_attributes(),
        }
    }
}

impl<'a, C0, D0, C1, D1, C2> IntoEntry for (May<(C0, D0)>, C1, D1, C2, &'a str) where
    C0: IntoMatch, D0: ToDirections, C1: IntoCurr, D1: ToDirections, C2: IntoMatch
{
    fn into_entry(self, text: &'static str) -> Entry {
        use self::Neighbor::{Must, May};
        Entry {
            instrumented: false,
            entry_text: text,
            loop_start: self.1.is_loop(),
            incoming: May((((self.0).0).0.into_match(),
                           ((self.0).0).1.to_directions())),
            curr: self.1.into_match(),
            outgoing: Must((self.2.to_directions(), self.3.into_match())),
            template: self.4.to_string(),
            include_attributes: vec![],
        }
    }
}

impl<'a, C0, D0, C1, D1, C2, A> IntoEntry for (May<(C0, D0)>, C1, D1, C2, &'a str, A) where
    C0: IntoMatch, D0: ToDirections, C1: IntoCurr, D1: ToDirections, C2: IntoMatch, A: IntoAttributes
{
    fn into_entry(self, text: &'static str) -> Entry {
        use self::Neighbor::{Must, May};
        Entry {
            instrumented: false,
            entry_text: text,
            loop_start: self.1.is_loop(),
            incoming: May((((self.0).0).0.into_match(),
                           ((self.0).0).1.to_directions())),
            curr: self.1.into_match(),
            outgoing: Must((self.2.to_directions(), self.3.into_match())),
            template: self.4.to_string(),
            include_attributes: self.5.into_attributes(),
        }
    }
}

impl<'a, C0, D0, C1, D1, C2> IntoEntry for (C0, D0, C1, May<(D1, C2)>, &'a str) where
    C0: IntoMatch, D0: ToDirections, C1: IntoCurr, D1: ToDirections, C2: IntoMatch
{
    fn into_entry(self, text: &'static str) -> Entry {
        use self::Neighbor::{Must, May};
        Entry {
            instrumented: false,
            entry_text: text,
            loop_start: self.2.is_loop(),
            incoming: Must((self.0.into_match(), self.1.to_directions())),
            curr: self.2.into_match(),
            outgoing: May((((self.3).0).0.to_directions(),
                           ((self.3).0).1.into_match())),
            template: self.4.to_string(),
            include_attributes: vec![],
        }
    }
}

impl<'a, C0, D0, C1, D1, C2, A> IntoEntry for (C0, D0, C1, May<(D1, C2)>, &'a str, A) where
    C0: IntoMatch, D0: ToDirections, C1: IntoCurr, D1: ToDirections, C2: IntoMatch, A: IntoAttributes
{
    fn into_entry(self, text: &'static str) -> Entry {
        use self::Neighbor::{Must, May};
        Entry {
            instrumented: false,
            entry_text: text,
            loop_start: self.2.is_loop(),
            incoming: Must((self.0.into_match(), self.1.to_directions())),
            curr: self.2.into_match(),
            outgoing: May((((self.3).0).0.to_directions(),
                           ((self.3).0).1.into_match())),
            template: self.4.to_string(),
            include_attributes: self.5.into_attributes(),
        }
    }
}

impl<'a, C0, D0, C1, D1, C2> IntoEntry for (May<(C0, D0)>, C1, May<(D1, C2)>, &'a str)
    where
    C0: IntoMatch, D0: ToDirections, C1: IntoCurr, D1: ToDirections, C2: IntoMatch
{
    fn into_entry(self, text: &'static str) -> Entry {
        use self::Neighbor::{May};
        Entry {
            instrumented: false,
            entry_text: text,
            loop_start: self.1.is_loop(),
            incoming: May((((self.0).0).0.into_match(), ((self.0).0).1.to_directions())),
            curr: self.1.into_match(),
            outgoing: May((((self.2).0).0.to_directions(), ((self.2).0).1.into_match())),
            template: self.3.to_string(),
            include_attributes: vec![],
        }
    }
}

impl<'a, C0, D0, C1, D1, C2, A> IntoEntry for (May<(C0, D0)>, C1, May<(D1, C2)>, &'a str, A)
    where
    C0: IntoMatch, D0: ToDirections, C1: IntoCurr, D1: ToDirections, C2: IntoMatch, A: IntoAttributes,
{
    fn into_entry(self, text: &'static str) -> Entry {
        use self::Neighbor::{May};
        Entry {
            instrumented: false,
            entry_text: text,
            loop_start: self.1.is_loop(),
            incoming: May((((self.0).0).0.into_match(), ((self.0).0).1.to_directions())),
            curr: self.1.into_match(),
            outgoing: May((((self.2).0).0.to_directions(), ((self.2).0).1.into_match())),
            template: self.3.to_string(),
            include_attributes: self.4.into_attributes(),
        }
    }
}

pub struct Start;
pub struct Finis;

impl<'a, C1, D1, C2> IntoEntry for (Start, C1, D1, C2, &'a str)
    where C1: IntoMatch, D1: ToDirections, C2: IntoMatch
{
    fn into_entry(self, text: &'static str) -> Entry {
        use self::Neighbor::{Blank, Must};
        Entry {
            instrumented: false,
            entry_text: text,
            loop_start: false,
            incoming: Blank,
            curr: self.1.into_match(),
            outgoing: Must((self.2.to_directions(), self.3.into_match())),
            template: self.4.to_string(),
            include_attributes: vec![]
        }
    }
}

impl<'a, C1, D1, C2, A> IntoEntry for (Start, C1, D1, C2, &'a str, A)
    where C1: IntoMatch, D1: ToDirections, C2: IntoMatch, A: IntoAttributes,
{
    fn into_entry(self, text: &'static str) -> Entry {
        use self::Neighbor::{Blank, Must};
        Entry {
            instrumented: false,
            entry_text: text,
            loop_start: false,
            incoming: Blank,
            curr: self.1.into_match(),
            outgoing: Must((self.2.to_directions(), self.3.into_match())),
            template: self.4.to_string(),
            include_attributes: self.5.into_attributes(),
        }
    }
}

impl<'a, C0, D0, C1> IntoEntry for (C0, D0, C1, Finis, &'a str)
    where C0: IntoMatch, D0: ToDirections, C1: IntoMatch
{
    fn into_entry(self, text: &'static str) -> Entry {
        use self::Neighbor::{Blank, Must};
        Entry {
            instrumented: false,
            entry_text: text,
            loop_start: false,
            incoming: Must((self.0.into_match(), self.1.to_directions())),
            curr: self.2.into_match(),
            outgoing: Blank,
            template: self.4.to_string(),
            include_attributes: vec![],
        }
    }
}

impl<'a, C0, D0, C1, A> IntoEntry for (C0, D0, C1, Finis, &'a str, A)
    where C0: IntoMatch, D0: ToDirections, C1: IntoMatch, A: IntoAttributes
{
    fn into_entry(self, text: &'static str) -> Entry {
        use self::Neighbor::{Blank, Must};
        Entry {
            instrumented: false,
            entry_text: text,
            loop_start: false,
            incoming: Must((self.0.into_match(), self.1.to_directions())),
            curr: self.2.into_match(),
            outgoing: Blank,
            template: self.4.to_string(),
            include_attributes: self.5.into_attributes(),
        }
    }
}

impl<'a, C1> IntoEntry for (All, C1, All, &'a str) where
    C1: IntoCurr,
{
    fn into_entry(self, text: &'static str) -> Entry {
        use self::Neighbor::{May};
        Entry {
            instrumented: false,
            entry_text: text,
            loop_start: self.1.is_loop(),
            incoming: May((Match::Any, directions::Any.to_directions())),
            curr: self.1.into_match(),
            outgoing: May((directions::Any.to_directions(), Match::Any)),
            template: self.3.to_string(),
            include_attributes: vec![],
        }
    }
}

impl<'a, C1, A> IntoEntry for (All, C1, All, &'a str, A) where
    C1: IntoCurr, A: IntoAttributes
{
    fn into_entry(self, text: &'static str) -> Entry {
        use self::Neighbor::{May};
        Entry {
            instrumented: false,
            entry_text: text,
            loop_start: self.1.is_loop(),
            incoming: May((Match::Any, directions::Any.to_directions())),
            curr: self.1.into_match(),
            outgoing: May((directions::Any.to_directions(), Match::Any)),
            template: self.3.to_string(),
            include_attributes: self.4.into_attributes(),
        }
    }
}

#[allow(dead_code)]
struct Loud<X>(X) where X: IntoEntry;
impl<X: IntoEntry> IntoEntry for Loud<X> {
    fn into_entry(self, text: &'static str) -> Entry {
        Entry { instrumented: true, ..self.0.into_entry(text) }
    }
}

macro_rules! entries {
    ($($e:expr),* $(,)*) => { vec![$($e.into_entry(stringify!($e)),)*] }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Table {
    pub(crate) entries: Vec<Entry>,
}

impl Table {
    pub fn demo() -> Self {
        use directions::{N, S, E, W, NE, SE, SW, NW};
        use directions::Any as AnyDir;
        Table {
            entries: entries! {
                ("|-/\\", AnyDir, Loop('+'), (N,S), "|", "M {C}"),
                ("|-/\\", AnyDir, Loop('+'), (E,W), "-", "M {C}"),

                (Start,   '-', (E,W), "-+", "M {RO} L {O}"),
                (Start,   '|', (N,S), "|+", "M {RO} L {O}"),
                (Start,   '+', AnyDir, Match::Any, "M {C}"),

                (Match::Any, (E,NE,N,NW,W), Loop('.'), (E,SE,S,SW,W), "-|\\/", "M {I} Q {C} {O}"),
                (Match::Any, (E,SE,S,SW,W), Loop('\''), (E,NE,N,NW,W), "-|\\/", "Q {C} {O}"),

                ("+-.'", (E, W), '-', May(((E, W), "-+.'>")), "L {O}"),
                ("+|.'", (N, S), '|', May(((N, S), "|+.'")), "L {O}"),

                (Match::Any, (E,NE,N,NW,W), '.', (E,SE,S,SW,W), "-|\\/", "Q {C} {O}"),
                (Match::Any, (E,SE,S,SW,W), '\'', (E,NE,N,NW,W), "-|\\/", "Q {C} {O}"),

                ("|-/\\>", AnyDir, '+', May(((N,S), "|")), "L {C}"),
                ("|-/\\>", AnyDir, '+', May(((E,W), "-")), "L {C}"),
                ("|-/\\>", AnyDir, '+', (NE,SW), "/", "L {C}"),
                ("|-/\\>", AnyDir, '+', (NW,SE), "\\", "L {C}"),

                (Match::Any, (NE, SW), '/', May(((NE, SW), "/+.'")), "L {O}"),
                (Match::Any, (NW, SE), '\\', May(((NW, SE), "\\+.'")), "L {O}"),

                ('-', E, '>', Finis, "L {C} l 3,0 m -3,-3 l 3,3 l -3,3 m 0,-3"),
                ('-', E, '>', E, '+', "L {E} m -2,0 l 4,0 m -4,-3 l 4,3 l -4,3 m 0,-3 m  4,0"),
                ('+', W, '>', W, '-', "M {E} m -2,0 l 4,0 m -4,-3 l 4,3 l -4,3 m 0,-3 m  4,0  M {E} L {C}"),

            }
        }
    }
}

impl Table {
    pub fn entries(&self) -> ::std::slice::Iter<Entry> { self.entries.iter() }

    pub(crate) fn find(&self,
                a: Announce,
                incoming: Option<(char, Direction)>,
                curr: char,
                outgoing: Option<(Direction, char)>) -> Option<(&str, &[(String, String)])> {
        for e in &self.entries {
            if !e.loop_start && e.matches(a, incoming, curr, outgoing) {
                return Some((&e.template, &e.include_attributes[..]));
            }
        }

        return None;
    }

    pub(crate) fn find_loop(&self,
                     a: Announce,
                     incoming: (char, Direction),
                     curr: char,
                     outgoing: (Direction, char)) -> Option<(&str, &[(String, String)])> {
        for e in &self.entries {
            if e.loop_start && e.matches(a, Some(incoming), curr, Some(outgoing)) {
                return Some((&e.template, &e.include_attributes[..]));
            }
        }

        return None;
    }
}
```

The template string is perhaps the most important part of the `format`
module. It is the domain-specific language for describing how to render
a given character.

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
impl Default for Table {
    fn default() -> Self {
        use directions::{N, S, E, W, NE, SE, SW, NW};
        use directions::Any as AnyDir;
        use directions::NonNorth;
        use directions::NonSouth;
        const JOINTS: &'static str = ".'+o";
        const LINES: &'static str = "-|/\\:=";
        const LINES_AND_JOINTS: &'static str = r"-|/\:=.'+o";
        const STRICT_LINES_AND_JOINTS: &'static str = r"-|/\:=+";
        const ZER_SLOPE: &'static str = r"-=.'+o><";
        const INF_SLOPE: &'static str = r"|:.'+o^v";
        const POS_SLOPE: &'static str =  r"/.'+o";
        const NEG_SLOPE: &'static str =  r"\.'+o";
        Table {
            entries: entries! {
                ('-', W, '-',  W, '-', ""),
                ('-', E, '-',  E, '-', ""),
                ('/', NE, '/', NE, '/', ""),
                ('/', SW, '/', SW, '/', ""),
                ('\\', NW, '\\', NW, '\\', ""),
                ('\\', SE, '\\', SE, '\\', ""),
                ('|', N, '|', N, '|', ""),
                ('|', S, '|', S, '|', ""),

                (Start, '-', E, Match::Any, "M {W} L {E}"),

                (Start, '-', W, Match::Any, "M {E} L {W}"),
                (Start, '|', N, Match::Any, "M {S} L {N}"),
                (Start, '|', S, Match::Any, "M {N} L {S}"),
                (Start, '/', (SW,S,W), Match::Any, "M {NE} L {SW}"),
                (Start, '/', (NE,N,E), Match::Any, "M {SW} L {NE}"),
                (Start,'\\', (SE,S,E), Match::Any, "M {NW} L {SE}"),
                (Start,'\\', (NW,N,W), Match::Any, "M {SE} L {NW}"),
                (Start, '.', (W,E), ZER_SLOPE, "M {S} Q {C} {O}"),
                (Start, '.', E, POS_SLOPE, "M {S}"),
                (Start, '.', W, NEG_SLOPE, "M {S}"),
                (Start, "'", (W,E), ZER_SLOPE, "M {N} Q {C} {O}"),
                (Start, "'", E, NEG_SLOPE, "M {N}"),
                (Start, "'", W, POS_SLOPE, "M {N}"),
```

This block adds support for little circles along a line,
via the elliptical arc command `A`.

```rust
                (STRICT_LINES_AND_JOINTS, AnyDir, 'o', Finis,
                 "L {I} A 2,2 360 1 0 {RI}  A 2,2 180 0 0 {I} M {RI}"),
```

Commented out code below is the same mistake I have
made elsewhere: there
are "natural" directions for characters like `/` and
`\`, which I have encoded in the SLOPE classes above.
But that means you cannot just match willy-nilly
against all LINES or LINES_AND_JOINTS in the
`next` component of the tuple; you need to put in
a stricter filter.

```rust
                // Loud((LINES_AND_JOINTS, AnyDir, 'o', AnyDir, LINES_AND_JOINTS,
                //      "L {I} A 2,2 360 1 0  {O}  A 2,2 180 0 0 {I} M {O}")),
                (LINES_AND_JOINTS, AnyDir, 'o', (W,E), r"-=+",
                      "L {I} A 2,2 360 1 0  {O}  A 2,2 180 0 0 {I} M {O}"),
                (LINES_AND_JOINTS, AnyDir, 'o', (N,S), r"|:+",
                      "L {I} A 2,2 360 1 0  {O}  A 2,2 180 0 0 {I} M {O}"),
                (LINES_AND_JOINTS, AnyDir, 'o', (NE,SW), r"/+",
                      "L {I} A 2,2 360 1 0  {O}  A 2,2 180 0 0 {I} M {O}"),
                (LINES_AND_JOINTS, AnyDir, 'o', (NW,SE), r"\+",
                      "L {I} A 2,2 360 1 0  {O}  A 2,2 180 0 0 {I} M {O}"),
```


This block is made of special cases for rendering horizontal
lines with curve characters in "interesting" ways.
They are not necessarily consistent nor do they exhibit symmetry,
but it seems better to do *something* rather than fall through
to default handlers that often show nothing special at all
along the path.
```rust
                (      r"\", E, '.', May((E, LINES)),   "Q {SW} {S}"),
                (      r"/", W, '.', May((W, LINES)),   "Q {SE} {S}"),
                (      r"/", E, "'", May((E, LINES)),   "Q {NW} {N}"),
                (      r"\", W, "'", May((W, LINES)),   "Q {NE} {N}"),
                (ZER_SLOPE, E, '.', May((E, LINES)),   "Q {C} {S}"),
                (ZER_SLOPE, W, '.', May((W, LINES)),   "Q {C} {S}"),
                (ZER_SLOPE, E, "'", May((E, LINES)),   "Q {C} {N}"),
                (ZER_SLOPE, W, "'", May((W, LINES)),   "Q {C} {N}"),
                (     ".'", E, '-', May((E, Match::Any)), "Q {W} {E}"),
                (     ".'", W, '-', May((W, Match::Any)), "Q {E} {W}"),
                (     ".",  E,  '/', May((E, r"'-\")), "Q {SW} {NE}"),
                (     ".",  W, r"\", May((W, r"'-/")), "Q {SE} {NW}"),
                (     "'",  E, r"\", May((E, r".-/")), "Q {NW} {SE}"),
                (     "'",  W, '/', May((W, r".-\")), "Q {NE} {SW}"),
```

These bits for `(` are another set of special cases for handling the
sides of a diamond when I don't want to use `+`.

By "diamond" I mean something like this:

```
  +    <-- `.` also acceptable here
 / \
(   )
 \ /
  +    <-- likewise `'` works here.
```

I don't want to use `+` here because I only want it to connect to the
diamond
and not to other neighboring lines (which is what `+` and other generic
joints would imply).

```rust
                // FIXME below cases seems like they are not always matching for some reason
                (     r"/", SW, '(', SE, r"\", "Q {C} {SE}"),
                (     r"/", NE, ')', NW, r"\", "Q {C} {NW}"),
                (     r"\", SE, ')', SW, r"/", "Q {C} {SW}"),
                (     r"\", NW, '(', NE, r"/", "Q {C} {NE}"),
                (Match::Any, AnyDir, r"/", SW, '(', "L {SW}"),
                (Match::Any, AnyDir, r"/", NE, ')', "L {NE}"),
                (Match::Any, AnyDir, r"\", SE, ')', "L {SE}"),
                (Match::Any, AnyDir, r"\", NW, '(', "L {NW}"),

                (Match::Any, E, '-', May((E, ZER_SLOPE)), "L {E}"),
                (Match::Any, W, '-', May((W, ZER_SLOPE)), "L {W}"),
                (Match::Any, N, '|', May((N, INF_SLOPE)), "L {N}"),
                (Match::Any, S, '|', May((S, INF_SLOPE)), "L {S}"),

                (Start, '=', E, ZER_SLOPE, "M {W} L {E}", [("stroke-dasharray", "5,2")]),
                (Start, '=', W, ZER_SLOPE, "M {E} L {W}", [("stroke-dasharray", "5,2")]),
                (Start, ':', N, INF_SLOPE, "M {S} L {N}", [("stroke-dasharray", "5,2")]),
                (Start, ':', S, INF_SLOPE, "M {N} L {S}", [("stroke-dasharray", "5,2")]),
                (Match::Any, E, '=', May((E, ZER_SLOPE)), "L {E}", [("stroke-dasharray", "5,2")]),
                (Match::Any, W, '=', May((W, ZER_SLOPE)), "L {W}", [("stroke-dasharray", "5,2")]),
                (Match::Any, N, ':', May((N, INF_SLOPE)), "L {N}", [("stroke-dasharray", "5,2")]),
                (Match::Any, S, ':', May((S, INF_SLOPE)), "L {S}", [("stroke-dasharray", "5,2")]),

                (Start, '+', AnyDir, Match::Any, "M {C}"),
                (Match::Any, AnyDir, '+', Finis, "L {C}"),
                // Below is riskier than I actually want to take
                // on right now.
                // (LINES_AND_JOINTS, AnyDir, '+', May((AnyDir, JOINTS)), "L {C}"),

                (Match::Any, NE, '/', May((NE, POS_SLOPE)), "L {NE}"),
                (Match::Any, SW, '/', May((SW, POS_SLOPE)), "L {SW}"),
                (Match::Any, SE, '\\', May((SE, NEG_SLOPE)), "L {SE}"),
                (Match::Any, NW, '\\', May((NW, NEG_SLOPE)), "L {NW}"),
                (Match::Any, NE, '/',  E, JOINTS, "L {NE}"),
                (Match::Any, SW, '/',  E, JOINTS, "L {NE}"),
                (Match::Any, SE, '\\', E, JOINTS, "L {SE}"),
                (Match::Any, NW, '\\', E, JOINTS, "L {SE}"),
                (Match::Any, NW, '\\', W, JOINTS, "L {NW}"),
                (Match::Any, SE, '\\', W, JOINTS, "L {NW}"),
                (Match::Any, NE, '/',  W, JOINTS, "L {SE}"),
                (Match::Any, SW, '/',  W, JOINTS, "L {SE}"),

                ('>', E, '+', May((AnyDir, LINES_AND_JOINTS)), "M {C}"),
                ('<', W, '+', May((AnyDir, LINES_AND_JOINTS)), "M {C}"),
                ('^', N, '+', May((AnyDir, LINES_AND_JOINTS)), "M {C}"),
                ('v', S, '+', May((AnyDir, LINES_AND_JOINTS)), "M {C}"),
                ("-=", (E, W), '+', May(((E, W), ZER_SLOPE)), "L {C}"),

                (LINES, AnyDir, Loop('+'), (N,S), INF_SLOPE, "M {C}"),
                (LINES, AnyDir, Loop('+'), (E,W), ZER_SLOPE, "M {C}"),
                (LINES, AnyDir, Loop('+'), (NE,SW), POS_SLOPE, "M {C}"),
                (LINES, AnyDir, Loop('+'), (NW,SE), NEG_SLOPE, "M {C}"),

                (LINES, AnyDir, '+', (N,S), INF_SLOPE, "L {C}"),
                (LINES, AnyDir, '+', (E,W), ZER_SLOPE, "L {C}"),
                (LINES, AnyDir, '+', (NE,SW), POS_SLOPE, "L {C}"),
                (LINES, AnyDir, '+', (NW,SE), NEG_SLOPE, "L {C}"),

                // The curves!  .-   .-  .-   .
                // part 1:      |   /     \  /| et cetera
                (Match::Any, NonSouth,      '.',  NonNorth, LINES, "Q {C} {O}"),
                (Match::Any, NonSouth, Loop('.'), NonNorth, LINES, "M {I} Q {C} {O}"),

                // curves       |   \/   /
                // part 2:      '-  '   '-   et cetera
                (Match::Any, NonNorth,      '\'',  NonSouth, LINES, "Q {C} {O}"),
                (Match::Any, NonNorth, Loop('\''), NonSouth, LINES, "M {I} Q {C} {O}"),

                // Arrow Heads!
                //
                // Perhaps more importantly, this code builds in an
                // assumption that each grid cell is 9x12 (or at least
                // WxH for W>9 and H>12).
                //
                // An assumption along these lines is perhaps
                // inevitable (I think its probably better to make
                // such an assumption up front rather than pretend
                // that the cell is a NxN square and thus have the
                // user be surprised when it turns out to be
                // non-square).
                //
                // But the question remains: is building in the
                // numbers 9 and 12 a good idea?  Or should they be
                // other numbers, like 3 and 4 (i.e. reduced form) or
                // 36 and 48 (which are both immediately divisible by
                // 2,3,4, and 6, which may be preferable to dealing in
                // fractions).
                //
                // horizontal arrow heads
                ('-', E, '>', Finis, "L {C} l 3,0 m -3,-3 l 3,3 l -3,3 m 0,-3"),
                (Start, '>', W, '-', "M {C} l 3,0 m -3,-3 l 3,3 l -3,3 m 0,-3"),
                ('-', W, '<', Finis, "L {C} l -3,0 m 3,-3 l -3,3 l 3,3 m 0,-3"),
                (Start, '<', E, '-', "M {C} l -3,0 m 3,-3 l -3,3 l 3,3 m 0,-3"),
                // vertical arrow heads
                (Start,  '^', S, '|', "M {C} l 0,-5 m -3,5 l 3,-5 l 3, 5 m -3,0"),
                (Start,  '^', S, ':', "M {C} l 0,-5 m -3,5 l 3,-5 l 3, 5 m -3,0", [("stroke-dasharray", "5,2")]),
                (Start,  'v', N, '|', "M {C} l 0,5 m -3,-5 l 3, 5 l 3,-5 m -3,0"),
                (Match::Any, S, 'v', Finis, "L {C} l 0,5 m -3,-5 l 3, 5 l 3,-5 m -3,0"),

                // arrow heads that join with other paths
                ('|', N, '^', N, '+', "L {N} l 0,-5 m -3,5 l 3,-5 l 3, 5 m -3,0 m 0,-5"),
                ('+', S, '^', S, '|', "M {N} l 0,-5 m -3,5 l 3,-5 l 3, 5 m -3,0 M {N} L {C}"),
                ('|', S, 'v', S, '+', "L {S} l 0,5 m -3,-5 l 3, 5 l 3,-5 m -3,0 m 0, 5"),
                ('+', N, 'v', N, '|', "L {S} l 0,5 m -3,-5 l 3, 5 l 3,-5 m -3,0 m 0, 5 M {S} L {C}"),
                ('-', E, '>', E, '+', "L {E} m -2,0 l 4,0 m -4,-3 l 4,3 l -4,3 m 0,-3 m  4,0"),
                ('+', W, '>', W, '-', "M {E} m -2,0 l 4,0 m -4,-3 l 4,3 l -4,3 m 0,-3 m  4,0  M {E} L {C}"),
                ('-', W, '<', W, '+', "L {W} m 2,0 l -4,0 m 4,-3 l -4,3 l 4,3 m 0,-3 m -4,0"),
                ('+', E, '<', E, '-', "M {W} m 2,0 l -4,0 m 4,-3 l -4,3 l 4,3 m 0,-3 m -4,0  M {W} L {C}"),

                (Start, '.', E, '-', "M {S} Q {C} {E}"),
                (Start, '.', W, '-', "M {S} Q {C} {W}"),
                (Start, '\'', E, '-', "M {N} Q {C} {E}"),
                (Start, '\'', W, '-', "M {N} Q {C} {W}"),
            }
        }
    }
}
```
