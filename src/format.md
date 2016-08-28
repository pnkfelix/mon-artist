```rust
use directions::{self, Direction, ToDirections};

#[derive(Clone)]
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
            Match::Any => true,
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

#[derive(Clone)]
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
#[derive(Clone)]
pub struct Entry {
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
}

impl Entry {
    pub(crate) fn matches_curr(&self, curr: char) -> bool {
        self.curr.matches(curr)
    }
    pub fn matches(&self,
                   incoming: Option<(char, Direction)>,
                   curr: char,
                   outgoing: Option<(Direction, char)>) -> bool {
        use self::Neighbor::{Blank, Must, May};
        match (&self.incoming, &incoming) {
            (&Blank, &Some(_)) | (&Must(..), &None) => return false,
            (&Blank, &None) | (&May(..), &None) => {}
            (&Must((ref m, ref dirs)), &Some((c, d))) |
            (&May((ref m, ref dirs)), &Some((c, d))) => {
                if !dirs.contains(&d) { return false; }
                if !m.matches(c) { return false; }
            }
        }

        if !self.curr.matches(curr) { return false; }

        match (&self.outgoing, &outgoing) {
            (&Blank, &Some(_)) | (&Must(..), &None) => return false,
            (&Blank, &None) | (&May(..), &None) => {}
            (&May((ref dirs, ref m)), &Some((d, c))) |
            (&Must((ref dirs, ref m)), &Some((d, c))) => {
                if !dirs.contains(&d) { return false; }
                if !m.matches(c) { return false; }
            }
        }

        return true;
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

pub trait IntoEntry { fn into_entry(self) -> Entry; }

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
    fn into_entry(self) -> Entry {
        use self::Neighbor::{Must};
        Entry {
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
    fn into_entry(self) -> Entry {
        use self::Neighbor::{Must};
        Entry {
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
    fn into_entry(self) -> Entry {
        use self::Neighbor::{Must, May};
        Entry {
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
    fn into_entry(self) -> Entry {
        use self::Neighbor::{Must, May};
        Entry {
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
    fn into_entry(self) -> Entry {
        use self::Neighbor::{Must, May};
        Entry {
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
    fn into_entry(self) -> Entry {
        use self::Neighbor::{Must, May};
        Entry {
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
    fn into_entry(self) -> Entry {
        use self::Neighbor::{May};
        Entry {
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
    fn into_entry(self) -> Entry {
        use self::Neighbor::{May};
        Entry {
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
    fn into_entry(self) -> Entry {
        use self::Neighbor::{Blank, Must};
        Entry {
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
    fn into_entry(self) -> Entry {
        use self::Neighbor::{Blank, Must};
        Entry {
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
    fn into_entry(self) -> Entry {
        use self::Neighbor::{Blank, Must};
        Entry {
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
    fn into_entry(self) -> Entry {
        use self::Neighbor::{Blank, Must};
        Entry {
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
    fn into_entry(self) -> Entry {
        use self::Neighbor::{May};
        Entry {
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
    fn into_entry(self) -> Entry {
        use self::Neighbor::{May};
        Entry {
            loop_start: self.1.is_loop(),
            incoming: May((Match::Any, directions::Any.to_directions())),
            curr: self.1.into_match(),
            outgoing: May((directions::Any.to_directions(), Match::Any)),
            template: self.3.to_string(),
            include_attributes: self.4.into_attributes(),
        }
    }
}

macro_rules! entries { ($($e:expr),* $(,)*) => { vec![$($e.into_entry(),)*] } }

#[allow(dead_code)]
#[derive(Clone)]
pub struct Table {
    pub(crate) entries: Vec<Entry>,
}

impl Table {
    pub fn find(&self,
                incoming: Option<(char, Direction)>,
                curr: char,
                outgoing: Option<(Direction, char)>) -> Option<(&str, &[(String, String)])> {
        for e in &self.entries {
            if e.matches(incoming, curr, outgoing) {
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

where a primitive point is either

* The center of the current grid cell, or
* One of the eight compass oriented extremities on the edge around
  the current grid cell.

(At some point I may add support for other primitive points, such
as points on the predecessor or successor grid cell. But for now
the intention is to only make it easy to describe paths relative
to the current grid cell.)

The syntax for specifying a placeholder value is bracket delimited.

For the nine primitive point cases (i.e. center or edge), one writes
one of the following as appropriate:

`{C}`, `{N}`, `{NE}`, `{E}`, `{SE}`, `{S}`, `{SW}, `{W}`, `{NW}`.

For a point along a line, one writes a decimal number in the range
[0,1] (followed by optional non-linebreak whitespace), followed by
two of the above base cases, delimited by a `-` mark (and again one
is allowed to include non-linebreak whitespace before and after the
`-`).

For example, the point that is 3/10 of the way along the path from
the center to the north-east corner could be written `{.3 C-NE}`.

The substituted value for the placeholder will be the absolute x,y
coordinates for the described point. Note that this means that one
should usually use the capital letter commands, which take absolute
coordinates as inputs, in tandem with placeholders.

```rust
impl Default for Table {
    fn default() -> Self {
        use directions::{N, S, E, W, NE, SE, SW, NW};
        use directions::Any as AnyDir;
        use directions::NonNorth;
        use directions::NonSouth;
        Table {
            entries: entries! {
                (Start, '-', E, Match::Any, "M {W} L {E}"),
                (Start, '-', W, Match::Any, "M {E} L {W}"),
                (Start, '|', N, Match::Any, "M {S} L {N}"),
                (Start, '|', S, Match::Any, "M {N} L {S}"),

                (Match::Any, E, '-', May((E, Match::Any)), "L {E}"),
                (Match::Any, W, '-', May((W, Match::Any)), "L {W}"),
                (Match::Any, N, '|', May((N, Match::Any)), "L {N}"),
                (Match::Any, S, '|', May((S, Match::Any)), "L {S}"),

                (Match::Any, AnyDir, Loop('+'), AnyDir, Match::Any, "M {C}"),

                // FIXME should these be included right now, in absence of
                // extension to augment attributes to switch to dashed
                // mode?
                (Start, '=', E, Match::Any, "M {W} L {E}", [("stroke-dasharray", "5,2")]),
                (Start, '=', W, Match::Any, "M {E} L {W}", [("stroke-dasharray", "5,2")]),
                (Start, ':', N, Match::Any, "M {S} L {N}", [("stroke-dasharray", "5,2")]),
                (Start, ':', S, Match::Any, "M {N} L {S}", [("stroke-dasharray", "5,2")]),
                (Match::Any, E, '=', May((E, Match::Any)), "L {E}", [("stroke-dasharray", "5,2")]),
                (Match::Any, W, '=', May((W, Match::Any)), "L {W}", [("stroke-dasharray", "5,2")]),
                (Match::Any, N, ':', May((N, Match::Any)), "L {N}", [("stroke-dasharray", "5,2")]),
                (Match::Any, S, ':', May((S, Match::Any)), "L {S}", [("stroke-dasharray", "5,2")]),

                (Start, '+', AnyDir, Match::Any, "M {C}"),

                (Match::Any, NE, '/', May((NE, Match::Any)), "L {NE}"),
                (Match::Any, SW, '/', May((SW, Match::Any)), "L {SW}"),
                (Match::Any, SE, '\\', May((SE, Match::Any)), "L {SE}"),
                (Match::Any, NW, '\\', May((NW, Match::Any)), "L {NW}"),
                ('>', E, '+', May((AnyDir, Match::Any)), "M {C}"),
                ('<', W, '+', May((AnyDir, Match::Any)), "M {C}"),
                ('^', N, '+', May((AnyDir, Match::Any)), "M {C}"),
                ('v', S, '+', May((AnyDir, Match::Any)), "M {C}"),
                (Match::Any, AnyDir, '+', May((AnyDir, Match::Any)), "L {C}"),

                // The curves!  .-   .-  .-
                // part 1:      |   /     \  et cetera
                //
                // FIXME: shouldn't the incoming edges be allowed to
                // be northern?  Seems like the main things to outlaw
                // are mirrors (which arguably should be built into
                // the engine, right?) and things *from* the north.
                (Match::Any, NonNorth, '.',  S, '|', "Q {C} {S}"),
                (Match::Any, NonNorth, '.', SE, '\\', "Q {C} {SE}"),
                (Match::Any, NonNorth, '.', SW, '/', "Q {C} {SW}"),
                (Match::Any, NonSouth, '.',  E, '-', "Q {C} {E}"),
                (Match::Any, NonSouth, '.',  W, '-', "Q {C} {W}"),

                // FIXME a potentially easy way to remove a lot of redundancy here:
                // add placeholders for incoming, outgoing, and reverses thereof.
                // E.g. `M {RI} Q {C} {O}`, so that incoming `NE` and outgoing `S`
                // yields `M {SW} {C} {S}`

                (Match::Any, NE, Loop('.'),  E, Match::Any, "M {SW} Q {C}  {E}"),
                (Match::Any, N,  Loop('.'),  E, Match::Any, "M {S}  Q {C}  {E}"),
                (Match::Any, NW, Loop('.'),  E, Match::Any, "M {SE} Q {C}  {E}"),
                (Match::Any,  E, Loop('.'), SE, Match::Any, "M  {W} Q {C} {SE}"),
                (Match::Any, NE, Loop('.'), SE, Match::Any, "M {SW} Q {C} {SE}"),
                (Match::Any,  N, Loop('.'), SE, Match::Any, "M  {S} Q {C} {SE}"),
                (Match::Any,  W, Loop('.'), SE, Match::Any, "M  {E} Q {C} {SE}"),
                (Match::Any,  E, Loop('.'), S, Match::Any, "M  {W} Q {C} {S}"),
                (Match::Any, NE, Loop('.'), S, Match::Any, "M {SW} Q {C} {S}"),
                (Match::Any, NW, Loop('.'), S, Match::Any, "M {SE} Q {C} {S}"),
                (Match::Any,  W, Loop('.'), S, Match::Any, "M  {E} Q {C} {S}"),
                (Match::Any,  N, Loop('.'), SW, Match::Any, "M  {S} Q {C} {SW}"),
                (Match::Any,  E, Loop('.'), SW, Match::Any, "M  {W} Q {C} {SW}"),
                (Match::Any, NW, Loop('.'), SW, Match::Any, "M {SE} Q {C} {SW}"),
                (Match::Any,  W, Loop('.'), SW, Match::Any, "M  {E} Q {C} {SW}"),

                // curves       |   \/   /
                // part 1:      '-  '   '-   et cetera
                (Match::Any, NonSouth, '\'', N, '|', "Q {C} {N}"),
                (Match::Any, NonSouth, '\'', NE, '/', "Q {C} {NE}"),
                (Match::Any, NonSouth, '\'', NW, '\\', "Q {C} {NW}"),
                (Match::Any, NonNorth, '\'', E, '-', "Q {C} {E}"),
                (Match::Any, NonNorth, '\'', W, '-', "Q {C} {W}"),

                // Arrow Heads! FIXME probably should just use opened arrow head
                // rather than the current closed ones.
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
                (Start, '>', E, '-', "M {C} l 3,0 m -3,-3 l 3,3 l -3,3 m 0,-3"),
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
