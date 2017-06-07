extern crate tango;

use std::env;

fn main() {
    // In general there are multiple code generating crates
    // (e.g. lalrpop as well as tango), and I do not want them to step
    // on each others toes.
    //
    // So I am segregating the tango source tree into its own
    // subdirectory (`lit/`). As an accident of history, I am
    // maintaining the `src` directory structure underneath that
    // (`lit/src/...`)

    let cwd = env::current_dir().unwrap();
    let lit = {
        let mut l = cwd.clone(); l.push("src/lit"); l
    };
    println!("lit: {:?}", lit);
    // env::set_current_dir(&lit).unwrap();
    tango::process_root().unwrap();
}
