#![feature(pub_restricted)]
#![feature(inclusive_range_syntax, inclusive_range)]
#![feature(type_ascription)]
#![feature(slice_patterns)]

/// (Macro imports need to appear at the crate root.)
#[macro_use] extern crate lazy_static;

#[macro_use]
extern crate log;

/// Literate programming stuff goes in `mod lit`.
mod lit;

use lit::{attrs};
pub use lit::{directions, find_text, find_path, format};
pub use lit::{grid, path};
pub use lit::{render, svg, text, test_data};

#[cfg(test)]
use lit::{env_logger};
use lit::{regex, treexml};

pub use lit::Scene;

/// LalrPop grammar and its generated goes in `mod grammar`.
mod grammar;
