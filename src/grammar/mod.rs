// Things that make you go "hmm"

use lalrpop_util;

#[allow(dead_code)]
mod rules;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Rule {
    pub pat: Match,
    pub render: Rendering,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Match {
    /// loop <prev> <dirs> <curr> <dirs> <next>
    Loop(CharSet, Dirs, CharSet, Dirs, CharSet),
    /// step <prev> <dirs> <curr> <dirs> <next>
    Step(CharSet, Dirs, CharSet, Dirs, CharSet),
    /// start             <curr> <dirs> <next>
    Start(CharSet, Dirs, CharSet),
    /// end <prev> <dir> <curr>
    End(CharSet, Dirs, CharSet),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Rendering {
    pub draw: String,
    pub attrs: Option<Vec<(String, String)>>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum CharSet {
    Char(char),
    String(String),
    Any,
}

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Unforgeable(UnusedMarker);

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct UnusedMarker;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Dir { N, NE, E, SE, S, SW, W, NW }

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Dirs(pub Vec<Dir>);

use self::Dir as D;

pub const ALL_DIRS: [Dir; 8] = [D::N, D::NE, D::E, D::SE, D::S, D::SW, D::W, D::NW];

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParseError<'input>(lalrpop_util::ParseError<usize, (usize, &'input str), ()>);

pub fn parse_rules(s: &str) -> Result<Vec<Rule>, ParseError> {
    rules::parse_Rules(s).map_err(|e| ParseError(e))
}

macro_rules! assert_ok {
    ($r: expr) => {
        let r;
        assert!({ r = $r; (r: Result<_, _>).is_ok() }, "result not ok; err: {:?}", r.unwrap_err());
    }
}


// Below is the sample grammar from PADL 2017 paper, re-adapted slight to the original
// syntax I had intended (where we do not have any \-escapes; instead you are intended
// to resort to raw-strings if that is actually necessary, which is not the case here
// because `"` is not a character we ever use in the diagram for that paper.
#[cfg(test)]
pub(crate) const SAMPLE_GRAMMAR: &'static str = r#"
loop  "|-/\" ANY '+' (N,S) "|" draw "M {C}";
loop  "|-/\" ANY '+' (E,W) "-" draw "M {C}";

# ‘-‘, ‘|‘, and ‘+‘ can start if next works. Draw line across.
start            '-' (E,W) "-+" draw "M {RO} L {O}";
start            '|' (N,S) "|+" draw "M {RO} L {O}";
start            '+' ANY   ANY  draw "M {C}";

# ‘.‘ and ‘’‘ make rounded corners. Draw curve through center.
step ANY (E,NE,N,NW,W) '.'  (E,SE,S,SW,W) "-|\/" draw "Q {C} {O}";
step ANY (E,SE,S,SW,W) "'" (E,NE,N,NW,W) "-|\/" draw "Q {C} {O}";

# ... for a loop, draw curve from incoming edge to outgoing one.
# loop ANY (E,NE,N,NW,W) '.'  (E,SE,S,SW,W) "-|\/" draw "M {I} Q {C} {O}";
# loop ANY (E,SE,S,SW,W) ''' (E,NE,N,NW,W) "-|\/" draw "M {I} Q {C} {O}";

# `-` and `|` connect w/ most things. Draw line to outgoing edge.
# step  "+-.'" (E, W)     '-'  (maybe (E, W)   "-+.'>") draw "L {O}";
# step  "+|.'" (N, S)     '|'  (maybe (N, S)   "|+.'" ) draw "L {O}";

# `+` is a corner; ensure compatible. Just draw line to center
# (the rest of corner is handled by next character, if present).
# step "|-/\>" ANY          '+'  (maybe (N,S) "|")     draw "L {C}";
# step "|-/\>" ANY          '+'  (maybe (E,W) "-")     draw "L {C}";
# step "|-/\>" ANY          '+'         (NE,SW) "/")   draw "L {C}";
# step "|-/\>" ANY          '+'         (NW,SE) "\")  draw "L {C}";

# `/`, `\` are diagonals. Draw line to outgoing corner.
# step ANY (NE, SW) '/'  (maybe (NE, SW) "/+.'")   draw "L {O}";
# step ANY (NW, SE) '\' (maybe (NW, SE) "\+.'")  draw "L {O}";

# Special case arrowhead code (1st does not touch; 2nd + 3rd do)
# end  '-' E '>'      draw "L {C} l 3,0 m -3,-3 l 3,3 l -3,3 m 0,-3";
# step '-' E '>' E '+' draw "L {E} m -2,0 l 4,0 m -4,-3 l 4,3 l -4,3 m 0,-3 m 4,0";
# step '+' W '>' W '-' draw "M {E} m -2,0 l 4,0 m -4,-3 l 4,3 l -4,3 m 0,-3 m 4,0 M {E} L {C}";
"#;

#[test]
fn sanity_check_1() {
    assert_ok!(parse_rules(r#"loop "|-/\" ANY "+" (N,S) "|" draw "M {C}"; "#));


    assert_ok!(parse_rules(SAMPLE_GRAMMAR));
}

#[test]
fn are_attributes_supported() {
    assert_ok!(parse_rules(r#"
start '^' (S) ':' draw "M {C} l 0,-5 m -3,5 l 3,-5 l 3, 5 m -3,0" attrs [("stroke-dasharray", "5,2")];
"#));
}
