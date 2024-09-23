//! Taken from `tests/ui/dyn-star/box.rs`.
//!
//! Originally:
//!
//! ```
//! //@ run-pass
//! //@ revisions: current next
//! //@ ignore-compare-mode-next-solver (explicit revisions)
//! //@[current] compile-flags: -C opt-level=0
//! //@[next] compile-flags: -Znext-solver -C opt-level=0
//! ```
//!
//! Which might look like the following in structured TOML approach:
//!
//! ```compiletest
//! mode = "run-pass"
//! revisions = ["current", "next"]
//!
//! [[ignore]]
//! compare-mode = "next-solver"
//! reason = "explicit revisions"
//!
//! # default solver
//! [revision."current"]
//! compile-flags = ["-C opt-level=0"]
//!
//! # next solver
//! [revision."next"]
//! compile-flags = ["-Znext-solver", "-C opt-level=0"]
//! ```

#![feature(dyn_star)]
#![allow(incomplete_features)]

use std::fmt::Display;

fn make_dyn_star() -> dyn* Display {
    Box::new(42) as dyn* Display
}

fn main() {
    let x = make_dyn_star();

    println!("{x}");
}
