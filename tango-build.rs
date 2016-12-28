extern crate tango;
extern crate lalrpop;

fn main() {
    // I do not want lalrpop and tango to step on each others toes.
    // So we will segregate the two source trees.

    // lalrpop::process_root().unwrap();
    tango::process_root().unwrap();
}
