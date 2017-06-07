Steps to reproduce:

```
% cargo build --verbose
   Compiling reduction v0.1.1 (file:///Users/fklock/Dev/Rust/reduction)
     Running `rustc --crate-name build_script_tango_build tango-build.rs --crate-type bin --emit=dep-info,link -C debuginfo=2 -C metadata=62717369972352f4 -C extra-filename=-62717369972352f4 --out-dir /Users/fklock/Dev/Rust/reduction/target/debug/build/reduction-62717369972352f4 -L dependency=/Users/fklock/Dev/Rust/reduction/target/debug/deps`
     Running `/Users/fklock/Dev/Rust/reduction/target/debug/build/reduction-62717369972352f4/build-script-tango-build`
     Running `rustc --crate-name mon_artist src/lib.rs --crate-type lib --emit=dep-info,link -C debuginfo=2 -C metadata=f443131a805a9905 -C extra-filename=-f443131a805a9905 --out-dir /Users/fklock/Dev/Rust/reduction/target/debug/deps -L dependency=/Users/fklock/Dev/Rust/reduction/target/debug/deps`
    Finished dev [unoptimized + debuginfo] target(s) in 0.48 secs
% touch tango-build.rs
% cargo build --verbose
   Compiling reduction v0.1.1 (file:///Users/fklock/Dev/Rust/reduction)
     Running `rustc --crate-name build_script_tango_build tango-build.rs --crate-type bin --emit=dep-info,link -C debuginfo=2 -C metadata=62717369972352f4 -C extra-filename=-62717369972352f4 --out-dir /Users/fklock/Dev/Rust/reduction/target/debug/build/reduction-62717369972352f4 -L dependency=/Users/fklock/Dev/Rust/reduction/target/debug/deps`
     Running `/Users/fklock/Dev/Rust/reduction/target/debug/build/reduction-62717369972352f4/build-script-tango-build`
     Running `rustc --crate-name mon_artist src/lib.rs --crate-type lib --emit=dep-info,link -C debuginfo=2 -C metadata=f443131a805a9905 -C extra-filename=-f443131a805a9905 --out-dir /Users/fklock/Dev/Rust/reduction/target/debug/deps -L dependency=/Users/fklock/Dev/Rust/reduction/target/debug/deps`
    Finished dev [unoptimized + debuginfo] target(s) in 0.47 secs
% touch src/lit/src/mod.md
% cargo build --verbose
   Compiling reduction v0.1.1 (file:///Users/fklock/Dev/Rust/reduction)
     Running `/Users/fklock/Dev/Rust/reduction/target/debug/build/reduction-62717369972352f4/build-script-tango-build`
     Running `rustc --crate-name mon_artist src/lib.rs --crate-type lib --emit=dep-info,link -C debuginfo=2 -C metadata=f443131a805a9905 -C extra-filename=-f443131a805a9905 --out-dir /Users/fklock/Dev/Rust/reduction/target/debug/deps -L dependency=/Users/fklock/Dev/Rust/reduction/target/debug/deps`
    Finished dev [unoptimized + debuginfo] target(s) in 0.15 secs
% touch src/lit/src/mod.rs
% cargo build --verbose
   Compiling reduction v0.1.1 (file:///Users/fklock/Dev/Rust/reduction)
     Running `rustc --crate-name mon_artist src/lib.rs --crate-type lib --emit=dep-info,link -C debuginfo=2 -C metadata=f443131a805a9905 -C extra-filename=-f443131a805a9905 --out-dir /Users/fklock/Dev/Rust/reduction/target/debug/deps -L dependency=/Users/fklock/Dev/Rust/reduction/target/debug/deps`
    Finished dev [unoptimized + debuginfo] target(s) in 0.14 secs
```
