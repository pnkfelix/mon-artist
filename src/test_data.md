```rust
pub const ALL: [(&'static str, &'static str); 29] = [
    BASIC,
    BASIC_NAMED_RECT,
    BASIC_NAMED_CLOSED,
    BASIC_NAMED_UNCLOSED,
    BASIC_NAMED_UNCLOSED_PREFIX,
    BASIC_ATTRS,
    LINE,
    LINE_WITH_ID,
    VERT_LINE,
    VERT_LINE_WITH_ID,
    TEXT_WITH_ATTRS,
    DASHED,
    VERT_DASH,
    CURVES,

    LEFT_ARROW,
    RIGHT_ARROW,
    BASIC_ALL_PLUS,
    BASIC_UL_PLUS,

    ARROW_POINT_SOUTH_AT_BOX,
    ARROW_POINT_SOUTH_TO_BOX,
    DOUBLE_ARROW_POINT_SOUTH_AT_BOX,
    DOUBLE_ARROW_POINT_SOUTH_TO_BOX,
    BOX_AT_BOX,
    BOX_TO_BOX,
    RBOX_AT_RBOX,
    RBOX_TO_RBOX,

    MULTI_RECTS,

    ISSUE_15_DESC,

    EXPERIMENTS
        ];

macro_rules! def_test {
    ($NAME:ident, $data: expr) => {
        pub const $NAME: (&'static str, &'static str) = (stringify!($NAME), $data);
    }
}

def_test! { LINE, r#"
 above
------
 below
"# }

def_test! { LINE_WITH_ID, r#"
 above
------[line_id]
 below
"# }

def_test! { VERT_LINE, r#"
| top
|
| bottom
"# }

def_test! { VERT_LINE_WITH_ID, r#"
| top
|
| bottom
|[line_id]
"# }

def_test! { TEXT_WITH_ATTRS, r#"
hello world
[h]

[h]: font-style='italic'
"# }

def_test! { DASHED, r#"
 above
-=-=-=
 below
"# }

def_test! { VERT_DASH, r#"
: top
|
: bottom
"# }

def_test! { CURVES, r#"
--.  +--+      +--.      .----+  +---------.  .------+
  |  |   \    /    \    /    /    \       /    \     |
  +--'    +--+      '--'    +------+     '------'    end
"# }
// 00000011111111112222222222333333333344444444445555555555
// 45678901234567890123456789012345678901234567890123456789

def_test! { LEFT_ARROW, r#"
 above
<-----
 below
"# }

def_test! { RIGHT_ARROW, r#"
above
----->
below
"# }

def_test! { BASIC,
            ".----.  top\n\
             |    |\n\
             '----'  bottom\n" }
//           00000000011111
//           12345678901234

pub const BASIC_WIDTH: u32 = 14;
pub const BASIC_HEIGHT: u32 = 3;

def_test! { BASIC_WO_BOX,
            "______  top\n\
             _    _\n\
             ______  bottom\n" }

def_test! { BASIC_NAMED_RECT,
            ".----.  top\n\
             |[b] |\n\
             '----'  bottom\n\
                           \n\
             [b]: fill='yellow'\n" }
//           00000000011111
//           12345678901234

def_test! { BASIC_NAMED_CLOSED,
            ".----.  top\n\
             |[b] |\n\
             '----+  bottom\n\
                           \n\
             [b]: fill='yellow'\n" }
//           00000000011111
//           12345678901234


def_test! { BASIC_NAMED_UNCLOSED,
            r#"
.----.  top
     |
    -+  bottom
"# }
//           00000000011111
//           12345678901234

def_test! { BASIC_NAMED_UNCLOSED_PREFIX,
            r#"
.----.  top
|[b] |
'-  -+  bottom
^
| We used to omit steps before top-left corner here
  [note]

[b]: stroke='blue'
[note]: font-size='10' font-style='italic' font-family='Trattatello'
"# }
//           00000000011111
//           12345678901234


def_test! { BASIC_UL_PLUS,
            "+----.  top\n\
             |    |\n\
             '----'  bottom\n" }
//           00000000011111
//           12345678901234

def_test! { BASIC_UR_PLUS,
            ".----+  top\n\
             |[b] |\n\
             '----'  bottom\n" }
//           00000000011111
//           12345678901234

def_test! { BASIC_ALL_PLUS,
            "+----+  top\n\
             |[b] |\n\
             +----+  bottom\n" }
//           00000000011111
//           12345678901234

def_test! { BASIC_ATTRS,
            ".----.  top\n\
             |[b] |\n\
             '----'  bottom\n\
             [b]: fill='yellow'\n" }
//           00000000011111
//           12345678901234


def_test! { ARROW_POINT_SOUTH_AT_BOX,
            "|   |[a]  top\n\
             |   |\n\
             |   v\n\
             | +----+\n\
             | |[b] |\n\
             | +----+  bottom\n" }
//           00000000011111
//           12345678901234


def_test! { ARROW_POINT_SOUTH_TO_BOX,
            "|   |[a]  top\n\
             |   |\n\
             |   v\n\
             | +-+--+\n\
             | |[b] |\n\
             | +----+  bottom\n" }
//           00000000011111
//           12345678901234


def_test! { DOUBLE_ARROW_POINT_SOUTH_AT_BOX,
            "|   ^     top\n\
             |   |\n\
             |   v\n\
             | +----+\n\
             | |    |\n\
             | +----+  bottom\n" }
//           00000000011111
//           12345678901234


def_test! { BOX_AT_BOX,
            "() +---+ top    \n\
             () |   |        \n\
             () +---+[t]     \n\
             ()   ^          \n\
             ()   |          \n\
             ()   v          \n\
             () +---+[b]     \n\
             () |   |        \n\
             () +---+ bottom \n" }
//           00000000011111111
//           12345678901234567

def_test! { BOX_TO_BOX,
            "() +---+ top    \n\
             () |   |        \n\
             () +-+-+[t]     \n\
             ()   ^          \n\
             ()   |          \n\
             ()   v          \n\
             () +-+-+[b]     \n\
             () |   |        \n\
             () +---+ bottom \n" }
//           00000000011111111
//           12345678901234567

def_test! { RBOX_AT_RBOX,
            "() .---. top    \n\
             () |   |        \n\
             () '---'[t]     \n\
             ()   ^          \n\
             ()   |          \n\
             ()   v          \n\
             () .---.[b]     \n\
             () |   |        \n\
             () '---' bottom \n" }
//           00000000011111111
//           12345678901234567

def_test! { RBOX_TO_RBOX,
            "() .---. top    \n\
             () |   |        \n\
             () '-+-'[t]     \n\
             ()   ^          \n\
             ()   |          \n\
             ()   v          \n\
             () .-+-.[b]     \n\
             () |   |        \n\
             () '---' bottom \n" }
//           00000000011111111
//           12345678901234567

def_test! { DOUBLE_ARROW_POINT_SOUTH_TO_BOX,
            "|   ^     top\n\
             |   |\n\
             |   v\n\
             | +-+--+\n\
             | |[b] |\n\
             | +----+  bottom\n" }
//           00000000011111
//           12345678901234


def_test! { MULTI_RECTS,
            // 111122222222223333333333444444444455555555556
            // 678901234567890123456789012345678901234567890
            r#"
.-------------.
|[a]          |
|   A Box!    |<----.   .-------------.
|             |     |   |[c]          |
'-------------'     |   |   Another   |---------+
      ^             '-->|     Box     |         |
      :                 |             |         |
      |                 '-------------'         |
      v                    |                    |
    .-+-.[b]          o ---+                    |
    |   |                  | .-.                |
    '---'                  | | |  +--------------------+
                           '-' |  |[d]                 |
                               |  :                    |
                               '->+                    |
                                  |                    |
                                  +--------------------+
"# }

def_test! { ISSUE_15_DESC,
            // 111122222222223333333333444444444455555555556
            // 678901234567890123456789012345678901234567890
        r#"
.-------------.
|[a]          |
|   A Box!    +<----.   .-------------.
|             |     |   |[c]          |
'-----+-------'     |   |   Another   +---------+
      ^             '-->+     Box     |         |
      |                 |             |         |
      |                 '-------------'         |
      v                                         |
    .-+-.[b]                                    |
    |   |                                       |
    '---'                         +-------------+------+
                                  |[d]                 |
                                  |                    |
                                  |                    |
                                  |                    |
                                  +--------------------+
"# }

def_test! { A2S_LOGO,

            r#"
                      .-------------------------.
                      |                         |
                      | .---.-. .-----. .-----. |
                      | | .-. | +-->  | |  <--| |
                      | | '-' | |  <--| +-->  | |
                      | '---'-' '-----' '-----' |
                      |  ascii     2      svg   |
                      |                         |
                      '-------------------------'
                       https://9vx.org/~dho/a2s/

"# }

def_test! { EXPERIMENTS,
            // 111122222222223333333333444444444455555555556
            // 678901234567890123456789012345678901234567890
        r#"
-.- -./ -./.\.- -././.-

-'- -'\ -'\'/'- -'\'\'-

.----.---.
|         \
'-.        +------+
   \       |
    '------+

       +-----+       +-----+
       |  A  |       |  A  |
       +--+--+       +--.--+
         / \           / \
        +-+-+         +---+
          |             |
   +------+------+    Other
   |             |
+--+--+       +--+--+
|  B  |       |  C  |
+-----+       +-----+

+                           +
 \   +   \         +   /   /
  )   \   )       /   (   /
 /     \ /       (     \ /
        '         \     '

+        +     \           +          +
 \        \     \         /    /     /
  \        \     )       /    /     /
   )        \   /       (    (     /
  /          \ /         \    \   /
 /            '           \    \ /
                                '

      /          \    |  |   \|   \ /
     o-  o-  -o  -o  -o  o-   o    o
          \  /

        |  \      /
  +-o-  o   o    o   -o  o-   o    o   -o  o
        |    \  /     |  |   /|   / \      |
        +                     +  +         +


      /          \    |  |   \|   \ /
     O-  O-  -O  -O  -O  O-   O    O
          \  /

        |  \      /
  +-O-  O   O    O   -O  O-   O    O   -O  O
        |    \  /     |  |   /|   / \      |
        +                     +  +         +

                  \|  |/
               \   O  O   /
               -O        O-
             /    |    |    \
           -O    -O    O-    O-

           -O    -O    O-    O-
             \    |    |    /
               -O        O-
               /   O  O   \
                  /|  |\


.-------------.
|[a]          |
|   A Box!    |<----.   .-------------.
|             |     |   |[c]          |
'-------------'     |   |   Another   |---------+  O-----o
      ^             '-->|     Box     |         | / \   /|
      :                 |             |         |/   \ / |
      |                 '-------------'   O-----+     o  O
      v                    |                    |    /  /
    .-+-.[b]          o ---+              O ----+      /
    |   |                  | .-.                |
    '---'                  | | |  +--------------------+
                           '-' |  |[d]                 |
                               |  :                    |
                               '->+                    |
                                  |                    |
                                  +--------------------+


   +-----+   .
  /     /   / \
 +-----+   /   \
    ^     /     \      +---------------+
    |    / flow  \     |               |
    '---( charts  )--->| Ask Questions |
         \       /     |               |
          \     /      +-------.-------+
           \   /                o
            \ /               \ | /
             '                 \|/   .------.
                                +    | o o  |
                               / \   |  ^   |
   uh, oh                     /   \  | '-'  |
                                     '---+--'
      o      o     o                     |
     \|/     |     |     o             --+--
      +      +    -+-   -+-              |
     / \    / \   / \   / \              +
                                        / \
                                       /   \

 +---+  +-o-+  o-o-o  o-+-+
 |   |  |   |  |   |  |\ \|
 +---+  +---+  o---o  +-+-o
  [a]    [b]    [c]    [d]

[a]: fill='blue'
[b]: fill='brown'
[c]: fill='cyan' fillrule='evenodd'
[d]: fill='yellow'
"# }
```
