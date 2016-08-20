```rust
pub const ALL: [(&'static str, &'static str); 16] = [
    BASIC,
    LINE,
    VERT_LINE,
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
    MULTI_RECTS,

    ISSUE_15_DESC
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

def_test! { VERT_LINE, r#"
| top
|
| bottom
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

def_test! { BASIC_UL_PLUS,
            "+----.  top\n\
             |    |\n\
             '----'  bottom\n" }
//           00000000011111
//           12345678901234

def_test! { BASIC_UR_PLUS,
            ".----+  top\n\
             |    |\n\
             '----'  bottom\n" }
//           00000000011111
//           12345678901234

def_test! { BASIC_ALL_PLUS,
            "+----+  top\n\
             |    |\n\
             +----+  bottom\n" }
//           00000000011111
//           12345678901234


def_test! { ARROW_POINT_SOUTH_AT_BOX,
            "  |     top\n\
             ()|\n\
             ()V\n\
             +----+\n\
             |    |\n\
             +----+  bottom\n" }
//           00000000011111
//           12345678901234


def_test! { ARROW_POINT_SOUTH_TO_BOX,
            "  |     top\n\
             ()|\n\
             ()V\n\
             +-+--+\n\
             |    |\n\
             +----+  bottom\n" }
//           00000000011111
//           12345678901234


def_test! { DOUBLE_ARROW_POINT_SOUTH_AT_BOX,
            "  ^     top\n\
             ()|\n\
             ()V\n\
             +----+\n\
             |    |\n\
             +----+  bottom\n" }
//           00000000011111
//           12345678901234


def_test! { DOUBLE_ARROW_POINT_SOUTH_TO_BOX,
            "  ^     top\n\
             ()|\n\
             ()V\n\
             +-+--+\n\
             |    |\n\
             +----+  bottom\n" }
//           00000000011111
//           12345678901234


def_test! { MULTI_RECTS,
            // 111122222222223333333333444444444455555555556
            // 678901234567890123456789012345678901234567890
            r#"
.-------------.
|             |
|   A Box!    |<----.   .-------------.
|             |     |   |             |
'-------------'     |   |   Another   |---------+
      ^             '-->|     Box     |         |
      |                 |             |         |
      |                 '-------------'         |
      v                                         |
    .---.                                       |
    |   |                                       |
    '---'                         +--------------------+
                                  |                    |
                                  |                    |
                                  |                    |
                                  |                    |
                                  +--------------------+
"# }

def_test! { ISSUE_15_DESC,
            // 111122222222223333333333444444444455555555556
            // 678901234567890123456789012345678901234567890
        r#"
.-------------.
|             |
|   A Box!    +<----.   .-------------.
|             |     |   |             |
'-----+-------'     |   |   Another   +---------+
      ^             '-->+     Box     |         |
      |                 |             |         |
      |                 '-------------'         |
      v                                         |
    .-+-.                                       |
    |   |                                       |
    '---'                         +-------------+------+
                                  |                    |
                                  |                    |
                                  |                    |
                                  |                    |
                                  +--------------------+
"# }
```
