use std::process::Command;

fn main() {
    Command::new("touch").arg("src/lit/src/mod.rs").spawn().unwrap();
}
