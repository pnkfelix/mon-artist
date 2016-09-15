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
    for argument in env::args().skip(1) {
        println!("processing {}", argument);
        process(&argument).unwrap();
    }
    println!("Hello World 2");
}

#[derive(Debug)]
enum Error {
    IO(io::Error),
    Parse(ParseError),
}

impl From<io::Error> for Error { fn from(e: io::Error) -> Self { Error::IO(e) } }
impl From<ParseError> for Error { fn from(e: ParseError) -> Self { Error::Parse(e) } }

fn process(argument: &str) -> Result<(), Error> {
    let mut input = File::open(argument)?;
    let mut content = String::new();
    input.read_to_string(&mut content)?;
    let s = content.parse::<Grid>()?.into_scene(&Default::default());
    let r = SvgRender {
        x_scale: 8, y_scale: 13,
        font_family: "monospace".to_string(), font_size: 13,
        show_gridlines: false,
        name: argument.to_string(),
    };
    let svg = r.render_s(&s);
    let mut output = File::create(format!("{}.svg", argument))?;
    Ok(write!(&mut output, "{}", svg)?)
}
```
