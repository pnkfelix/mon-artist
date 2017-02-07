extern crate mon_artist;

use mon_artist::render::svg::{SvgRender};
use mon_artist::render::{RenderS};
use mon_artist::grid::{Grid, ParseError};
use mon_artist::{SceneOpts};

use std::convert::From;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};

fn main() {
    let mut args = env::args();
    args.next(); // skip program name
    loop {
        let table = if let Some(arg) = args.next() { arg } else { break; };
        let in_file =  if let Some(arg) = args.next() { arg } else { break; };
        let out_file =  if let Some(arg) = args.next() { arg } else { break; };
        println!("processing {} to {} via {}", in_file, out_file, table);
        process(&table, &in_file, &out_file).unwrap();
    }
    // describe_table();
}

#[allow(dead_code)]
fn describe_table() {
    use mon_artist::format::{Table, Neighbor};

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

use mon_artist::format::Table;

fn get_table(table: &str) -> Table {
    match File::open(table) {
        Ok(input) => Table::from_lines(BufReader::new(input).lines()),
        Err(err) => match table {
            "default" => Table::default(),
            "demo"    => Table::demo(),
            _ => panic!("Unknown table name: {}, file err: {:?}", table, err),
        },
    }
}

fn process(table: &str, in_file: &str, out_file: &str) -> Result<(), Error> {
    let mut input = File::open(in_file)?;
    let mut content = String::new();
    input.read_to_string(&mut content)?;
    let table = get_table(table);
    let s = content.parse::<Grid>()?.into_scene(
        &table,
        Some(SceneOpts { text_infer_id: false, ..Default::default() }));
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
