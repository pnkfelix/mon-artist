```rust
pub const DEFAULT_INPUT: &'static str = r#"
start '-' (E) ANY draw "M {W} L {E}";
start '-' (W) ANY draw "M {E} L {W}";

start '|' (N) ANY draw "M {S} L {N}";
start '|' (S) ANY draw "M {N} L {S}";

start  '/' (SW,S,W) ANY draw "M {NE} L {SW}";
start  '/' (NE,N,E) ANY draw "M {SW} L {NE}";
start  '\' (SE,S,E) ANY draw "M {NW} L {SE}";
start  '\' (NW,N,W) ANY draw "M {SE} L {NW}";
start  '.' (W,E)    "-=.'+oO><" draw "M {S} Q {C} {O}";
start  '.' (E)      "/.'+oO" draw "M {S}";
start  '.' (W)      "\.'+oO" draw "M {S}";
start  "'" (W,E)    "-=.'+oO><" draw "M {N} Q {C} {O}";
start  "'" (E)      "\.'+oO" draw "M {N}";
start  "'" (W)      "/.'+oO" draw "M {N}";

# This block adds support for little circles along a line,
# via the elliptical arc command `A`.

end   "-|/\:=+" ANY 'o' draw "L {I/o} A 2,2 0 1 0 {RI/o}  A 2,2 0 0 0 {I/o} A 2,2 0 1 0 {RI/o}";

# Commented out code below is the same mistake I have made elsewhere:
# there are "natural" directions for characters like `/` and `\`,
# which I have encoded in the SLOPE classes above.  But that means you
# cannot just match willy-nilly against all LINES or LINES_AND_JOINTS
# in the `next` component of the tuple; you need to put in a stricter
# filter.

# Loud(("-|/\:=.'+oO", AnyDir, 'o', AnyDir, "-|/\:=.'+oO",
#       "L {I} A 4,4 360 1 0  {O}  A 4,4 180 0 0 {I} M {O}")),

step "-|/\:=.'+oO" ANY 'o' (W,E)   "-=+" draw "L {I/o} A 2,2  0 1 0  {O/o}  A 2,2  0 0 0 {I/o} A 2,2 0 1 0 {O/o}";
step "-|/\:=.'+oO" ANY 'o' (N,S)   "|:+" draw "L {I/o} A 2,2  0 1 0  {O/o}  A 2,2  0 0 0 {I/o} A 2,2 0 1 0 {O/o}";
step "-|/\:=.'+oO" ANY 'o' (NE,SW) "/+"  draw "L {I/o} A 2,2  0 1 0  {O/o}  A 2,2  0 0 0 {I/o} A 2,2 0 1 0 {O/o}";
step "-|/\:=.'+oO" ANY 'o' (NW,SE) "\+"  draw "L {I/o} A 2,2  0 1 0  {O/o}  A 2,2  0 0 0 {I/o} A 2,2 0 1 0 {O/o}";
loop "-|/\:=.'+oO" ANY 'o' (W,E)   "-=+" draw "M {I} L {I/o} A 2,2 0 1 0  {O/o}  A 2,2 0 0 0 {I/o} A 2,2 0 1 0 {O/o}";
loop "-|/\:=.'+oO" ANY 'o' (N,S)   "|:+" draw "M {I} L {I/o} A 2,2 0 1 0  {O/o}  A 2,2 0 0 0 {I/o} A 2,2 0 1 0 {O/o}";
loop "-|/\:=.'+oO" ANY 'o' (NE,SW) "/+"  draw "M {I} L {I/o} A 2,2 0 1 0  {O/o}  A 2,2 0 0 0 {I/o} A 2,2 0 1 0 {O/o}";
loop "-|/\:=.'+oO" ANY 'o' (NW,SE) "\+"  draw "M {I} L {I/o} A 2,2 0 1 0  {O/o}  A 2,2 0 0 0 {I/o} A 2,2 0 1 0 {O/o}";

# These are similar to all the circle rules above, but the above rules will sometimes
# yield small circles when joining pair of lines at a thin angle. Sometimes that's
# the right thing, but when you want the circle sizes to be regular, you can use this
# instead.

end  "-|/\:=+" ANY 'O' draw "L {I/o} A 4,4 0 1 0 {RI/o}  A 4,4 0 0 0 {I/o} A 4,4 0 1 0 {RI/o}";

step "-|/\:=.'+oO" ANY 'O' (W,E) "-=+"  draw "L {I/o} A 4,4  0 1 0  {O/o}  A 4,4  0 0 0 {I/o} A 4,4 0 1 0 {O/o}";
step "-|/\:=.'+oO" ANY 'O' (N,S) "|:+"  draw "L {I/o} A 4,4  0 1 0  {O/o}  A 4,4  0 0 0 {I/o} A 4,4 0 1 0 {O/o}";
step "-|/\:=.'+oO" ANY 'O' (NE,SW) "/+" draw "L {I/o} A 4,4  0 1 0  {O/o}  A 4,4  0 0 0 {I/o} A 4,4 0 1 0 {O/o}";
step "-|/\:=.'+oO" ANY 'O' (NW,SE) "\+" draw "L {I/o} A 4,4  0 1 0  {O/o}  A 4,4  0 0 0 {I/o} A 4,4 0 1 0 {O/o}";

loop "-|/\:=.'+oO" ANY 'O' (W,E)   "-=+"  draw "M {I} L {I/o} A 4,4 0 1 0  {O/o}  A 4,4 0 0 0 {I/o} A 4,4 0 1 0 {O/o}";
loop "-|/\:=.'+oO" ANY 'O' (N,S)   "|:+"  draw "M {I} L {I/o} A 4,4 0 1 0  {O/o}  A 4,4 0 0 0 {I/o} A 4,4 0 1 0 {O/o}";
loop "-|/\:=.'+oO" ANY 'O' (NE,SW) "/+"   draw "M {I} L {I/o} A 4,4 0 1 0  {O/o}  A 4,4 0 0 0 {I/o} A 4,4 0 1 0 {O/o}";
loop "-|/\:=.'+oO" ANY 'O' (NW,SE) "\+"   draw "M {I} L {I/o} A 4,4 0 1 0  {O/o}  A 4,4 0 0 0 {I/o} A 4,4 0 1 0 {O/o}";

# This block is made of special cases for rendering horizontal
# lines with curve characters in "interesting" ways.
# They are not necessarily consistent nor do they exhibit symmetry,
# but it seems better to do *something* rather than fall through
# to default handlers that often show nothing special at all
# along the path.

step       "\"   (E) '.'  (E) "-|/\:="  draw "Q {SW} {S}";
end        "\"   (E) '.'                draw "Q {SW} {S}";
step       "/"   (W) '.'  (W) "-|/\:="  draw "Q {SE} {S}";
end        "/"   (W) '.'                draw "Q {SE} {S}";
step       "/"   (E) "'"  (E) "-|/\:="  draw "Q {NW} {N}";
end        "/"   (E) "'"                draw "Q {NW} {N}";
step       "\"   (W) "'"  (W) "-|/\:="  draw "Q {NE} {N}";
end        "\"   (W) "'"                draw "Q {NE} {N}";
step "-=.'+oO><" (E) '.'  (E) "-|/\:="  draw "Q {C} {S}";
end  "-=.'+oO><" (E) '.'                draw "Q {C} {S}";
step "-=.'+oO><" (W) '.'  (W) "-|/\:="  draw "Q {C} {S}";
end  "-=.'+oO><" (W) '.'                draw "Q {C} {S}";
step "-=.'+oO><" (E) "'"  (E) "-|/\:="  draw "Q {C} {N}";
end  "-=.'+oO><" (E) "'"                draw "Q {C} {N}";
step "-=.'+oO><" (W) "'"  (W) "-|/\:="  draw "Q {C} {N}";
end  "-=.'+oO><" (W) "'"                draw "Q {C} {N}";
step      ".'"   (E) '-'  (E) ANY       draw "Q {W} {E}";
end       ".'"   (E) '-'                draw "Q {W} {E}";
step      ".'"   (W) '-'  (W) ANY       draw "Q {E} {W}";
end       ".'"   (W) '-'                draw "Q {E} {W}";
step      "."    (E) '/'  (E) "'-\"     draw "Q {SW} {NE}";
end       "."    (E) '/'                draw "Q {SW} {NE}";
step      "."    (W) "\"  (W) "'-/"     draw "Q {SE} {NW}";
end       "."    (W) "\"                draw "Q {SE} {NW}";
step      "'"    (E) "\"  (E) ".-/"     draw "Q {NW} {SE}";
end       "'"    (E) "\"                draw "Q {NW} {SE}";
step      "'"    (W) '/'  (W) ".-\"     draw "Q {NE} {SW}";
end       "'"    (W) '/'                draw "Q {NE} {SW}";

# These bits for `(` are another set of special cases for handling the
# sides of a diamond when I don't want to use `+`.
#
# By "diamond" I mean something like this:
#
# ```
#    +    <-- `.` also acceptable here
#   / \
#  (   )
#   \ /
#    +    <-- likewise `'` works here.
# ```
#
# I don't want to use `+` here because I only want it to connect to the
# diamond
# and not to other neighboring lines (which is what `+` and other generic
# joints would imply).

# FIXME below cases seems like they are not always matching for some reason
step "/" (SW) '(' (SE) "\" draw "Q {C} {SE}";
step "/" (NE) ')' (NW) "\" draw "Q {C} {NW}";
step "\" (SE) ')' (SW) "/" draw "Q {C} {SW}";
step "\" (NW) '(' (NE) "/" draw "Q {C} {NE}";
step ANY ANY  "/" (SW) '(' draw "L {SW}";
step ANY ANY  "/" (NE) ')' draw "L {NE}";
step ANY ANY  "\" (SE) ')' draw "L {SE}";
step ANY ANY  "\" (NW) '(' draw "L {NW}";

step ANY (E) '-' (E) "-=.'+oO><" draw "L {E}";
end  ANY (E) '-'                 draw "L {E}";
step ANY (W) '-' (W) "-=.'+oO><" draw "L {W}";
end  ANY (W) '-'                 draw "L {W}";
step ANY (N) '|' (N) "|:.'+oO^v" draw "L {N}";
end  ANY (N) '|'                 draw "L {N}";
step ANY (S) '|' (S) "|:.'+oO^v" draw "L {S}";
end  ANY (S) '|'                 draw "L {S}";

start         '='       (E) "-=.'+oO><" draw "M {W} L {E}" attrs [("stroke-dasharray", "5,2")];
start         '='       (W) "-=.'+oO><" draw "M {E} L {W}" attrs [("stroke-dasharray", "5,2")];
start         ':'       (N) "|:.'+oO^v" draw "M {S} L {N}" attrs [("stroke-dasharray", "5,2")];
start         ':'       (S) "|:.'+oO^v" draw "M {N} L {S}" attrs [("stroke-dasharray", "5,2")];
step  ANY (E) '='       (E) "-=.'+oO><" draw "L {E}" attrs [("stroke-dasharray", "5,2")];
end   ANY (E) '='                       draw "L {E}" attrs [("stroke-dasharray", "5,2")];
step  ANY (W) '='       (W) "-=.'+oO><" draw "L {W}" attrs [("stroke-dasharray", "5,2")];
end   ANY (W) '='                       draw "L {W}" attrs [("stroke-dasharray", "5,2")];
step  ANY (N) ':'       (N) "|:.'+oO^v" draw "L {N}" attrs [("stroke-dasharray", "5,2")];
end   ANY (N) ':'                       draw "L {N}" attrs [("stroke-dasharray", "5,2")];
step  ANY (S) ':'       (S) "|:.'+oO^v" draw "L {S}" attrs [("stroke-dasharray", "5,2")];
end   ANY (S) ':'                       draw "L {S}" attrs [("stroke-dasharray", "5,2")];

start         '+' ANY ANY draw "M {C}";
end   ANY ANY '+'         draw "L {C}";

# # Below is riskier than I actually want to take
# # on right now.
# ("-|/\:=.'+oO" ANY '+', May((AnyDir, ".'+oO")), "L {C}"),

step ANY (NE) '/' (NE) "/.'+oO" draw "L {NE}";
end  ANY (NE) '/'               draw "L {NE}";
step ANY (SW) '/' (SW) "/.'+oO" draw "L {SW}";
end  ANY (SW) '/'               draw "L {SW}";
step ANY (SE) '\' (SE) "\.'+oO" draw "L {SE}";
end  ANY (SE) '\'               draw "L {SE}";
step ANY (NW) '\' (NW) "\.'+oO" draw "L {NW}";
end  ANY (NW) '\'               draw "L {NW}";

step ANY (NE) '/' (E) ".'+oO" draw "L {NE}";
step ANY (SW) '/' (E) ".'+oO" draw "L {NE}";
step ANY (SE) '\' (E) ".'+oO" draw "L {SE}";
step ANY (NW) '\' (E) ".'+oO" draw "L {SE}";
step ANY (NW) '\' (W) ".'+oO" draw "L {NW}";
step ANY (SE) '\' (W) ".'+oO" draw "L {NW}";
step ANY (NE) '/' (W) ".'+oO" draw "L {SE}";
step ANY (SW) '/' (W) ".'+oO" draw "L {SE}";

step                 '>'  (E)   '+' ANY   "-|/\:=.'+oO" draw "M {C}";
end                  '>'  (E)   '+'                     draw "M {C}";
step                 '<'  (W)   '+' ANY   "-|/\:=.'+oO" draw "M {C}";
end                  '<'  (W)   '+'                     draw "M {C}";
step                 '^'  (N)   '+' ANY   "-|/\:=.'+oO" draw "M {C}";
end                  '^'  (N)   '+'                     draw "M {C}";
step                 'v'  (S)   '+' ANY   "-|/\:=.'+oO" draw "M {C}";
end                  'v'  (S)   '+'                     draw "M {C}";
step                 "-=" (E,W) '+' (E,W) "-=.'+oO><"   draw "L {C}";
end                  "-=" (E,W) '+'                     draw "L {C}";

loop "-|/\\:=" ANY '+' (N,S)   "|:.'+oO^v" draw "M {C}";
loop "-|/\\:=" ANY '+' (E,W)   "-=.'+oO><" draw "M {C}";
loop "-|/\\:=" ANY '+' (NE,SW) "/.'+oO"    draw "M {C}";
loop "-|/\\:=" ANY '+' (NW,SE) "\.'+oO"    draw "M {C}";

step "-|/\\:=" ANY '+' (N,S)   "|:.'+oO^v" draw "L {C}";
step "-|/\\:=" ANY '+' (E,W)   "-=.'+oO><" draw "L {C}";
step "-|/\\:=" ANY '+' (NE,SW) "/.'+oO"    draw "L {C}";
step "-|/\\:=" ANY '+' (NW,SE) "\.'+oO"    draw "L {C}";

# The curves!  .-   .-  .-   .
# part 1:      |   /     \  /| et cetera
step ANY (E,NE,N,NW,W)      '.' (E,SE,S,SW,W) "-|/\\:=" draw "Q {C} {O}";
loop ANY (E,NE,N,NW,W)      '.' (E,SE,S,SW,W) "-|/\\:=" draw "M {I} Q {C} {O}";

# curves       |   \/   /
# part 2:      '-  '   '-   et cetera
step ANY (E,SE,S,SW,W)      "'" (E,NE,N,NW,W) "-|/\\:=" draw "Q {C} {O}";
loop ANY (E,SE,S,SW,W)      "'" (E,NE,N,NW,W) "-|/\\:=" draw "M {I} Q {C} {O}";

## Arrow Heads!
##
## Perhaps more importantly, this code builds in an
## assumption that each grid cell is 9x12 (or at least
## WxH for W>9 and H>12).
##
## An assumption along these lines is perhaps
## inevitable (I think its probably better to make
## such an assumpt                    than pretend
## that the cell is a NxN square and thus have the
## user be surprised when it turns out to be
## non-square).
##
## But the question remains: is building in the
## numbers 9 and 12 a good idea?  Or should they be
## other numbers, like 3 and 4 (i.e. reduced form) or
## 36 and 48 (which are both immediately divisible by
## 2,3,4, and 6, which may be preferable to dealing in
## fractions).

# horizontal arrow heads
end   '-' (E) '>' draw "L {C} l 3,0 m -3,-3 l 3,3 l -3,3 m 0,-3";
start '>' (W) '-' draw "M {C} l 3,0 m -3,-3 l 3,3 l -3,3 m 0,-3";
end   '-' (W) '<' draw "L {C} l -3,0 m 3,-3 l -3,3 l 3,3 m 0,-3";
start '<' (E) '-' draw "M {C} l -3,0 m 3,-3 l -3,3 l 3,3 m 0,-3";

# vertical arrow heads
start '^' (S) '|' draw "M {C} l 0,-5 m -3,5 l 3,-5 l 3, 5 m -3,0";
start 'v' (N) '|' draw "M {C} l 0,5 m -3,-5 l 3, 5 l 3,-5 m -3,0";
end   ANY (S) 'v' draw "L {C} l 0,5 m -3,-5 l 3, 5 l 3,-5 m -3,0";
start '^' (S) ':' draw "M {C} l 0,-5 m -3,5 l 3,-5 l 3, 5 m -3,0" attrs [("stroke-dasharray", "5,2")];

# arrow heads that join with other paths
step '|' (N) '^' (N) '+' draw "L {N} l 0,-5 m -3,5 l 3,-5 l 3, 5 m -3,0 m 0,-5";
step '+' (S) '^' (S) '|' draw "M {N} l 0,-5 m -3,5 l 3,-5 l 3, 5 m -3,0 M {N} L {C}";
step '|' (S) 'v' (S) '+' draw "L {S} l 0,5 m -3,-5 l 3, 5 l 3,-5 m -3,0 m 0, 5";
step '+' (N) 'v' (N) '|' draw "L {S} l 0,5 m -3,-5 l 3, 5 l 3,-5 m -3,0 m 0, 5 M {S} L {C}";
step '-' (E) '>' (E) '+' draw "L {E} m -2,0 l 4,0 m -4,-3 l 4,3 l -4,3 m 0,-3 m  4,0";
step '+' (W) '>' (W) '-' draw "M {E} m -2,0 l 4,0 m -4,-3 l 4,3 l -4,3 m 0,-3 m  4,0  M {E} L {C}";
step '-' (W) '<' (W) '+' draw "L {W} m 2,0 l -4,0 m 4,-3 l -4,3 l 4,3 m 0,-3 m -4,0";
step '+' (E) '<' (E) '-' draw "M {W} m 2,0 l -4,0 m 4,-3 l -4,3 l 4,3 m 0,-3 m -4,0  M {W} L {C}";

start        '.'  (E) '-' draw "M {S} Q {C} {E}";
start        '.'  (W) '-' draw "M {S} Q {C} {W}";
start        "'" (E) '-' draw "M {N} Q {C} {E}";
start        "'" (W) '-' draw "M {N} Q {C} {W}";

"#;
```
