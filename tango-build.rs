extern crate tango;
extern crate lalrpop;

use std::env;

fn main() {
    // I do not want lalrpop and tango to step on each others toes.
    // So we will segregate the two source trees.

    let cwd = env::current_dir().unwrap();
    let grammar = {
        let mut g = cwd.clone(); g.push("src/grammar"); g
    };
    let lit = {
        let mut l = cwd.clone(); l.push("src/lit"); l
    };
    println!("grammar: {:?} lit: {:?}", grammar, lit);
    env::set_current_dir(&grammar).unwrap();
    env::set_current_dir(&lit).unwrap();

    env::set_current_dir(grammar).unwrap();
    lalrpop::process_root().unwrap();

    env::set_current_dir(&lit).unwrap();
    match tango::process_root() {
        Ok(_) => {}
        Err(e) => {
            println!("error: {:?}", e);
            panic!("error with tango::process_root in lit {}", lit.display());
        }
    }
}
