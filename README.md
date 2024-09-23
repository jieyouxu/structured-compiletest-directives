# Structured compiletest directives experiment

Current `compiletest` directives are rather unstructured, the syntax is unclear,
error messages are poor if they exist, and makes parsing and handling very
complicated in `compiletest`. In this repo, we experiment with a structured TOML
test metadata approach as proposed by Chris in
<https://rust-lang.zulipchat.com/#narrow/stream/131828-t-compiler/topic/Test.20header.20commands.20could.20just.20be.20toml.3F>.

This is merely for demonstration and exploration purposes, if we want to
actually make this change to `compiletest` we'll need both a MCP as well as a
solid migration plan for `//@`-style directives to the structured TOML
directives.

## Example

Before:

```rs
//@ run-pass
//@ revisions: current next
//@ ignore-compare-mode-next-solver (explicit revisions)
//@[current] compile-flags: -C opt-level=0
//@[next] compile-flags: -Znext-solver -C opt-level=0
```

After:

````rs
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
````
