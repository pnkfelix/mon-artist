```rust
pub const ALL: [&'static str; 12] = [
    BASIC,
    LINE.1,
    VERT_LINE,
    DASHED,
    VERT_DASH,
    CURVES,

    LEFT_ARROW,
    RIGHT_ARROW,
    BASIC_ALL_PLUS,
    BASIC_UL_PLUS,
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

pub const VERT_LINE: &'static str = r#"
| top
|
| bottom
"#;

pub const DASHED: &'static str = r#"
 above
-=-=-=
 below
"#;

pub const VERT_DASH: &'static str = r#"
: top
|
: bottom
"#;

pub const CURVES: &'static str = r#"
--.  +--+      +--.      .----+  +---------.  .------+
  |  |   \    /    \    /    /    \       /    \     |
  +--'    +--+      '--'    +------+     '------'    end
"#;

pub const LEFT_ARROW: &'static str = r#"
 above
<-----
 below
"#;

pub const RIGHT_ARROW: &'static str = r#"
above
----->
below
"#;

pub const BASIC: &'static str =
    ".----.  top\n\
     |    |\n\
     '----'  bottom\n";
//   00000000011111
//   12345678901234
pub const BASIC_WIDTH: u32 = 14;
pub const BASIC_HEIGHT: u32 = 3;

pub const BASIC_WO_BOX: &'static str =
    "\u{7f}\u{7f}\u{7f}\u{7f}\u{7f}\u{7f}  top\n\
     \u{7f}    \u{7f}\n\
     \u{7f}\u{7f}\u{7f}\u{7f}\u{7f}\u{7f}  bottom\n";

pub const BASIC_UL_PLUS: &'static str =
    "+----.  top\n\
     |    |\n\
     '----'  bottom\n";
//   00000000011111
//   12345678901234

pub const BASIC_UR_PLUS: &'static str =
    ".----+  top\n\
     |    |\n\
     '----'  bottom\n";
//   00000000011111
//   12345678901234

pub const BASIC_ALL_PLUS: &'static str =
    "+----+  top\n\
     |    |\n\
     +----+  bottom\n";
//   00000000011111
//   12345678901234

pub const MULTI_RECTS: &'static str =
    // 00111111111122222222223333333333444444444455555555556
    // 89012345678901234567890123456789012345678901234567890
        r#"\n
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
"#;

pub const ISSUE_15_DESC: &'static str =
    // 00111111111122222222223333333333444444444455555555556
    // 89012345678901234567890123456789012345678901234567890
        r#"\n
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
"#;
```
