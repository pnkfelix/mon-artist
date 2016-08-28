```rust
use directions::{Direction};
use grid::{Grid, Pt, DirVector};
use grid::{PtRangeIter, PtCharIntoIterator}; // import methods on Pt...Pt and (Pt,char)
use test_data::{BASIC, BASIC_WO_BOX, BASIC_UL_PLUS, BASIC_UR_PLUS, BASIC_ALL_PLUS};
use test_data::{ISSUE_15_DESC};
use path::{Path, Closed};
use super::{FindClosedPaths};

impl Path {
    fn closed(steps: Vec<(Pt, char)>) -> Path {
        Path { steps: steps, closed: Closed::Closed, id: None, attrs: None, }
    }

    fn open(steps: Vec<(Pt, char)>) -> Path {
        Path { steps: steps, closed: Closed::Open, id: None, attrs: None }
    }
}

#[test]
fn trivial_path_east() {
    let grid = "--- ".parse::<Grid>().unwrap();
    let opt_p = super::find_unclosed_path_from(&grid,
                                               &Default::default(),
                                               DirVector(Pt(1,1), Direction::E));
    assert_eq!(opt_p.unwrap(),
               Path::open((Pt(1,1)...Pt(3,1)).iter_char('-').collect()));
}

#[test]
fn trivial_path_west() {
    let grid = "--- ".parse::<Grid>().unwrap();
    let opt_p = super::find_unclosed_path_from(&grid,
                                               &Default::default(),
                                               DirVector(Pt(3,1), Direction::W));
    assert_eq!(opt_p.unwrap(),
               Path::open((Pt(3,1)...Pt(1,1)).iter_char('-').collect()));
}

#[test]
fn hopping_path_east() {
    // ::env_logger::init();
    let grid = "-+- ".parse::<Grid>().unwrap();
    let opt_p = super::find_unclosed_path_from(&grid,
                                               &Default::default(),
                                               DirVector(Pt(1,1), Direction::E));
```

The expected data could be written more succinctly like so:
`Path::open(vec![(Pt(1,1), '-'), (Pt(2,1), '+'), (Pt(3,1), '-')])`.
The reason I have chosen the more verbose iterator form is to
demonstrate that pattern on a simple example since we use
it extensively in the other tests that follow.

```rust
    assert_eq!(opt_p.unwrap(),
               Path::open((Pt(1,1), '-').into_iter()
                          .chain((Pt(2,1), '+').into_iter())
                          .chain((Pt(3,1), '-').into_iter())
                          .collect()));
}

#[test]
fn eastward_arrow() {
    let grid = "--> ".parse::<Grid>().unwrap();
    let opt_p = super::find_unclosed_path_from(&grid,
                                               &Default::default(),
                                               DirVector(Pt(1,1), Direction::E));
    assert_eq!(opt_p.unwrap(),
               Path::open((Pt(1,1)...Pt(2,1)).iter_char('-')
                          .chain((Pt(3,1), '>').into_iter())
                          .collect()));
}

#[test]
fn reverse_westward_arrow() {
    let grid = "<-- ".parse::<Grid>().unwrap();
    let opt_p = super::find_unclosed_path_from(&grid,
                                               &Default::default(),
                                               DirVector(Pt(1,1), Direction::E));
    assert_eq!(opt_p.unwrap(),
               Path::open((Pt(1,1), '<').into_iter()
                          .chain((Pt(2,1)...Pt(3,1)).iter_char('-'))
                          .collect()));
}

#[test]
fn westward_arrow() {
    let grid = "<-- ".parse::<Grid>().unwrap();
    let opt_p = super::find_unclosed_path_from(&grid,
                                               &Default::default(),
                                               DirVector(Pt(3,1), Direction::W));
    assert_eq!(opt_p.unwrap(),
               Path::open((Pt(3,1)...Pt(2,1)).iter_char('-')
                          .chain((Pt(1,1), '<').into_iter())
                          .collect()));
}

#[test]
fn reverse_eastward_arrow() {
    let grid = "--> ".parse::<Grid>().unwrap();
    let opt_p = super::find_unclosed_path_from(&grid,
                                               &Default::default(),
                                               DirVector(Pt(3,1), Direction::W));
    assert_eq!(opt_p.unwrap(),
               Path::open((Pt(3,1), '>').into_iter()
                          .chain((Pt(2,1)...Pt(1,1)).iter_char('-'))
                          .collect()));
}

#[test]
fn reverse_northward_arrow() {
    let grid = "^ \n| \n| ".parse::<Grid>().unwrap();
    let opt_p = super::find_unclosed_path_from(&grid,
                                               &Default::default(),
                                               DirVector(Pt(1,1), Direction::S));
    assert_eq!(opt_p.unwrap(),
               Path::open((Pt(1,1), '^').into_iter()
                          .chain((Pt(1,2)...Pt(1,3)).iter_char('|'))
                          .collect()));
}

#[test]
fn southward_arrow() {
    let grid = "| \n| \nv ".parse::<Grid>().unwrap();
    let opt_p = super::find_unclosed_path_from(&grid,
                                               &Default::default(),
                                               DirVector(Pt(1,1), Direction::S));
    assert_eq!(opt_p.unwrap(),
               Path::open((Pt(1,1)...Pt(1,2)).iter_char('|')
                          .chain((Pt(1,3), 'v').into_iter())
                          .collect()));
}

#[test]
fn reverse_southward_arrow() {
    let grid = "| \n| \nv ".parse::<Grid>().unwrap();
    let opt_p = super::find_unclosed_path_from(&grid,
                                               &Default::default(),
                                               DirVector(Pt(1,3), Direction::N));
    assert_eq!(opt_p.unwrap(),
               Path::open((Pt(1,3), 'v').into_iter()
                          .chain((Pt(1,2)...Pt(1,1)).iter_char('|'))
                          .collect()));
}

#[test]
fn eastward_arrow_to_joiner() {
    let grid = "-->+ ".parse::<Grid>().unwrap();
    let opt_p = super::find_unclosed_path_from(&grid,
                                               &Default::default(),
                                               DirVector(Pt(1,1), Direction::E));
    assert_eq!(opt_p.unwrap(),
               Path::open((Pt(1,1)...Pt(2,1)).iter_char('-')
                          .chain((Pt(3,1), '>').into_iter())
                          .chain((Pt(4,1), '+').into_iter())
                          .collect()));
}

#[test]
fn double_vertical_arrow_to_joiners() {
    let input = r#"
+
^
|
v
+
"#;
    let grid = input.parse::<Grid>().unwrap();
    let opt_p = super::find_unclosed_path_from(&grid,
                                               &Default::default(),
                                               DirVector(Pt(1,2), Direction::S));
    assert_eq!(opt_p.unwrap(),
               Path::open((Pt(1,2), '+').into_iter()
                          .chain((Pt(1,3), '^').into_iter())
                          .chain((Pt(1,4), '|').into_iter())
                          .chain((Pt(1,5), 'v').into_iter())
                          .chain((Pt(1,6), '+').into_iter())
                          .collect()));
}

#[test]
fn double_vertical_arrow_to_used_joiners() {
    let input = r#"
+
^
|
v
+
"#;
    let mut grid = input.parse::<Grid>().unwrap();
    grid.mark_used(Pt(1,2));
    grid.mark_used(Pt(1,6));
    let opt_p = super::find_unclosed_path_from(&grid,
                                               &Default::default(),
                                               DirVector(Pt(1,2), Direction::S));
    assert_eq!(opt_p.unwrap(),
               Path::open((Pt(1,2), '+').into_iter()
                          .chain((Pt(1,3), '^').into_iter())
                          .chain((Pt(1,4), '|').into_iter())
                          .chain((Pt(1,5), 'v').into_iter())
                          .chain((Pt(1,6), '+').into_iter())
                          .collect()));
}
```


Several of the tests involve boxes. Here is a helper
routine that, given the four corners of a box, makes
the path for that box. (These are "simple boxes" because
only their corners can be customized, not their edges.)

```rust
fn simple_box_path(ul: (Pt, char), ur: (Pt, char),
            bl: (Pt, char), br: (Pt, char)) -> Path {
    Path::closed(ul.into_iter()
                 .chain((ul.0.e()..ur.0).iter_char('-'))
                 .chain(ur.into_iter())
                 .chain((ur.0.s()..br.0).iter_char('|'))
                 .chain(br.into_iter())
                 .chain((br.0.w()..bl.0).iter_char('-'))
                 .chain(bl.into_iter())
                 .chain((bl.0.n()..ul.0).iter_char('|'))
                 .collect())
}

#[test]
fn basic_single_box_upper_left() {
    let grid = BASIC.1.parse::<Grid>().unwrap();
    let opt_p = {
        let mut pf = FindClosedPaths::new(&grid);
        pf.find_closed_path(Pt(1,1))
    };
    assert_eq!(opt_p.clone().unwrap(),
               simple_box_path((Pt(1,1), '.'), (Pt(6,1), '.'),
                               (Pt(1,3), '\''), (Pt(6,3), '\'')));
    let mut grid = grid;
    grid.remove_path(&opt_p.unwrap());
    assert_eq!(grid.to_string(), BASIC_WO_BOX.1);
}

#[test]
fn basic_ul_plus_single_box_upper_left() {
    let grid = BASIC_UL_PLUS.1.parse::<Grid>().unwrap();
    let mut pf = FindClosedPaths::new(&grid);
    let opt_p = pf.find_closed_path(Pt(1,1));
    assert_eq!(opt_p.unwrap(),
               simple_box_path((Pt(1,1), '+'), (Pt(6,1), '.'),
                               (Pt(1,3), '\''), (Pt(6,3), '\'')));
}

#[test]
fn basic_ur_plus_single_box_upper_left() {
    // ::env_logger::init();
    let grid = BASIC_UR_PLUS.1.parse::<Grid>().unwrap();
    let mut pf = FindClosedPaths::new(&grid);
    let opt_p = pf.find_closed_path(Pt(1,1));
    assert_eq!(opt_p.unwrap(),
               simple_box_path((Pt(1,1), '.'), (Pt(6,1), '+'),
                               (Pt(1,3), '\''), (Pt(6,3), '\'')));
}

#[test]
fn basic_all_plus_single_box_upper_left() {
    let grid = BASIC_ALL_PLUS.1.parse::<Grid>().unwrap();
    let mut pf = FindClosedPaths::new(&grid);
    let opt_p = pf.find_closed_path(Pt(1,1));
    assert_eq!(opt_p.unwrap(),
               simple_box_path((Pt(1,1), '+'), (Pt(6,1), '+'),
                               (Pt(1,3), '+'), (Pt(6,3), '+')));
}

#[test]
fn issue_15_box_big_upper_left() {
    let grid = ISSUE_15_DESC.1.parse::<Grid>().unwrap();
    let mut pf = FindClosedPaths::new(&grid);
    let opt_p = pf.find_closed_path(Pt(1, 2));
    assert_eq!(opt_p.unwrap(),
               Path::closed(vec![(Pt(1,2), '.'), (Pt(2,2), '-'), (Pt(3,2), '-'), (Pt(4,2), '-'),
                                 (Pt(5,2), '-'), (Pt(6,2), '-'), (Pt(7,2), '-'), (Pt(8,2), '-'),
                                 (Pt(9,2), '-'), (Pt(10,2), '-'), (Pt(11,2), '-'), (Pt(12,2), '-'),
                                 (Pt(13,2), '-'), (Pt(14,2), '-'), (Pt(15,2), '.'),
                                 (Pt(15,3), '|'),
                                 (Pt(15,4), '+'),
                                 (Pt(15,5), '|'),
                                 (Pt(15,6), '\''), (Pt(14,6), '-'), (Pt(13,6), '-'), (Pt(12,6), '-'),
                                 (Pt(11,6), '-'), (Pt(10,6), '-'), (Pt(9,6), '-'), (Pt(8,6), '-'),
                                 (Pt(7,6), '+'), (Pt(6,6), '-'), (Pt(5,6), '-'), (Pt(4,6), '-'),
                                 (Pt(3,6), '-'), (Pt(2,6), '-'), (Pt(1,6), '\''),
                                 (Pt(1,5), '|'),
                                 (Pt(1,4), '|'),
                                 (Pt(1,3), '|')]));
}

#[test]
fn issue_15_box_lil_lower_left() {
    let grid = ISSUE_15_DESC.1.parse::<Grid>().unwrap();
    let mut pf = FindClosedPaths::new(&grid);
    let opt_p = pf.find_closed_path(Pt(5, 11));
    assert_eq!(opt_p.unwrap(),
               Path::closed(vec![(Pt(5,11), '.'), (Pt(6,11), '-'), (Pt(7,11), '+'), (Pt(8,11), '-'), (Pt(9,11), '.'),
                                 (Pt(9,12), '|'),
                                 (Pt(9,13), '\''), (Pt(8,13), '-'), (Pt(7,13), '-'), (Pt(6,13), '-'), (Pt(5,13), '\''),
                                 (Pt(5,12), '|')]));
}

#[test]
fn issue_15_box_big_upper_middle() {
    // ::env_logger::init();
    let grid = ISSUE_15_DESC.1.parse::<Grid>().unwrap();
    let mut pf = FindClosedPaths::new(&grid);
    let opt_p = pf.find_closed_path(Pt(25, 4));
    assert_eq!(opt_p.unwrap(),
               Path::closed(vec![(Pt(25,4), '.'), (Pt(26,4), '-'), (Pt(27,4), '-'), (Pt(28,4), '-'), (Pt(29,4), '-'),
                                 (Pt(30,4), '-'), (Pt(31,4), '-'), (Pt(32,4), '-'), (Pt(33,4), '-'), (Pt(34,4), '-'),
                                 (Pt(35,4), '-'), (Pt(36,4), '-'), (Pt(37,4), '-'), (Pt(38,4), '-'), (Pt(39,4), '.'),
                                 (Pt(39,5), '|'),
                                 (Pt(39,6), '+'),
                                 (Pt(39,7), '|'),
                                 (Pt(39,8), '|'),
                                 (Pt(39,9), '\''), (Pt(38,9), '-'), (Pt(37,9), '-'), (Pt(36,9), '-'), (Pt(35,9), '-'),
                                 (Pt(34,9), '-'), (Pt(33,9), '-'), (Pt(32,9), '-'), (Pt(31,9), '-'), (Pt(30,9), '-'),
                                 (Pt(29,9), '-'), (Pt(28,9), '-'), (Pt(27,9), '-'), (Pt(26,9), '-'), (Pt(25,9), '\''),
                                 (Pt(25,8), '|'),
                                 (Pt(25,7), '+'),
                                 (Pt(25,6), '|'),
                                 (Pt(25,5), '|')]));
}

#[test]
fn issue_15_box_big_lower_right() {
    // ::env_logger::init().ok(); // discard error since double-init is one.
    let grid = ISSUE_15_DESC.1.parse::<Grid>().unwrap();
    let mut pf = FindClosedPaths::new(&grid);
    let opt_p = pf.find_closed_path(Pt(35, 13));
    assert_eq!(opt_p.unwrap(),
               Path::closed(vec![(Pt(35,13), '+'), (Pt(36,13), '-'), (Pt(37,13), '-'), (Pt(38,13), '-'), (Pt(39,13), '-'),
                                 (Pt(40,13), '-'), (Pt(41,13), '-'), (Pt(42,13), '-'), (Pt(43,13), '-'), (Pt(44,13), '-'),
                                 (Pt(45,13), '-'), (Pt(46,13), '-'), (Pt(47,13), '-'), (Pt(48,13), '-'), (Pt(49,13), '+'),
                                 (Pt(50,13), '-'), (Pt(51,13), '-'), (Pt(52,13), '-'), (Pt(53,13), '-'), (Pt(54,13), '-'),
                                 (Pt(55,13), '-'), (Pt(56,13), '+'),
                                 (Pt(56,14), '|'),
                                 (Pt(56,15), '|'),
                                 (Pt(56,16), '|'),
                                 (Pt(56,17), '|'),
                                 (Pt(56,18), '+'), (Pt(55,18), '-'), (Pt(54,18), '-'), (Pt(53,18), '-'), (Pt(52,18), '-'),
                                 (Pt(51,18), '-'), (Pt(50,18), '-'), (Pt(49,18), '-'), (Pt(48,18), '-'), (Pt(47,18), '-'),
                                 (Pt(46,18), '-'), (Pt(45,18), '-'), (Pt(44,18), '-'), (Pt(43,18), '-'), (Pt(42,18), '-'),
                                 (Pt(41,18), '-'), (Pt(40,18), '-'), (Pt(39,18), '-'), (Pt(38,18), '-'), (Pt(37,18), '-'),
                                 (Pt(36,18), '-'), (Pt(35,18), '+'),
                                 (Pt(35,17), '|'),
                                 (Pt(35,16), '|'),
                                 (Pt(35,15), '|'),
                                 (Pt(35,14), '|')]));
}

#[test]
fn trivial_box_removal() {
    let mut grid = ".--. top\n\
                    |  | mid\n\
                    '--' bot\n".parse::<Grid>().unwrap();
    let path = {
        let mut pf = FindClosedPaths::new(&grid);
        pf.find_closed_path(Pt(1,1)).unwrap()
    };
    grid.remove_path(&path);
    assert_eq!(grid.to_string(),
               "____ top\n\
                _  _ mid\n\
                ____ bot\n");
}

#[test]
fn box_removal_but_ul_corner() {
    let mut grid = "+--. top\n\
                    |  | mid\n\
                    '--' bot\n".parse::<Grid>().unwrap();
    let path = {
        let mut pf = FindClosedPaths::new(&grid);
        pf.find_closed_path(Pt(1,1)).unwrap()
    };
    grid.remove_path(&path);
    assert_eq!(grid.to_string(),
               "+___ top\n\
                _  _ mid\n\
                ____ bot\n");
}

#[test]
fn box_removal_but_ur_corner() {
    let mut grid = ".--+ top\n\
                    |  | mid\n\
                    '--' bot\n".parse::<Grid>().unwrap();
    let path = {
        let mut pf = FindClosedPaths::new(&grid);
        pf.find_closed_path(Pt(1,1)).unwrap()
    };
    grid.remove_path(&path);
    assert_eq!(grid.to_string(),
               "___+ top\n\
                _  _ mid\n\
                ____ bot\n");
}

#[test]
fn box_removal_but_bl_corner() {
    let mut grid = ".--. top\n\
                    |  | mid\n\
                    +--' bot\n".parse::<Grid>().unwrap();
    let path = {
        let mut pf = FindClosedPaths::new(&grid);
        pf.find_closed_path(Pt(1,1)).unwrap()
    };
    grid.remove_path(&path);
    assert_eq!(grid.to_string(),
               "____ top\n\
                _  _ mid\n\
                +___ bot\n");
}

#[test]
fn box_removal_but_br_corner() {
    let mut grid = ".--. top\n\
                    |  | mid\n\
                    '--+ bot\n".parse::<Grid>().unwrap();
    let path = {
        let mut pf = FindClosedPaths::new(&grid);
        pf.find_closed_path(Pt(1,1)).unwrap()
    };
    grid.remove_path(&path);
    assert_eq!(grid.to_string(),
               "____ top\n\
                _  _ mid\n\
                ___+ bot\n");
}
```
