```rust
#![feature(question_mark)]

extern crate mon_artiste;

use mon_artiste::render::svg::{SvgRender};
use mon_artiste::render::{RenderS};
use mon_artiste::grid::{Grid, ParseError};

use std::convert::From;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};

fn main() {
    let mut args = env::args();
    args.next(); // skip program name
    loop {
        let table = if let Some(arg) = args.next() { arg } else { break; };
        let in_file =  if let Some(arg) = args.next() { arg } else { break; };
        let out_file =  if let Some(arg) = args.next() { arg } else { break; };
        println!("processing {} to {}", in_file, out_file);
        process(&table, &in_file, &out_file).unwrap();
    }
    // describe_table();
}

#[allow(dead_code)]
fn describe_table() {
    use mon_artiste::format::{Table, Neighbor};

    let t: Table = Default::default();
    let mut may_both_count = 0;
    let mut may_start_count = 0;
    let mut may_end_count = 0;
    let mut neither_count = 0;
    for (j, e) in t.entries().enumerate() {
        println!("{} {:?}", j+1, e);
        match (&e.incoming(), &e.outgoing()) {
            (&Neighbor::May(..), &Neighbor::May(..)) => may_both_count += 1,
            (&Neighbor::May(..), _) => may_start_count += 1,
            (_, &Neighbor::May(..)) => may_end_count += 1,
            _ => neither_count += 1,
        }
    }
    println!("");
    println!("both: {} may_start: {} may_end: {} neither: {}",
             may_both_count, may_start_count, may_end_count, neither_count);
}

#[derive(Debug)]
enum Error {
    IO(io::Error),
    Parse(ParseError),
}

impl From<io::Error> for Error { fn from(e: io::Error) -> Self { Error::IO(e) } }
impl From<ParseError> for Error { fn from(e: ParseError) -> Self { Error::Parse(e) } }

use mon_artiste::format::Table;

fn get_table(table: &str) -> Table {
    match table {
        "default" => Table::default(),
        "demo"    => Table::demo(),
        _ => panic!("Unknown table name: {}", table),
    }
}

fn process(table: &str, in_file: &str, out_file: &str) -> Result<(), Error> {
    let mut input = File::open(in_file)?;
    let mut content = String::new();
    input.read_to_string(&mut content)?;
    let table = get_table(table);
    let s = content.parse::<Grid>()?.into_scene(&table);
    let r = SvgRender {
        x_scale: 8, y_scale: 13,
        font_family: "monospace".to_string(), font_size: 13,
        show_gridlines: false,
        infer_rect_elements: false,
        name: in_file.to_string(),
    };
    let svg = r.render_s(&s);
    let mut output = File::create(out_file)?;
    Ok(write!(&mut output, "{}", svg)?)
}
```
