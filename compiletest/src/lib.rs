//! The real `compiletest` has multiple steps and has to deal with many many quirks. This library
//! implements a very limited subset related to directive handling (including parsing and
//! validation), and only implements a very naive ui test mode and built upon `libtest` (not robust
//! at all!), since this is just for illustration purposes.
//!
//! # Key concepts
//!
//! ## Basic ideas
//!
//! - `compiletest` **directives** are special metadata written in test source files (we're
//!   expecting Rust source files only, but adaptations are possible depending on future needs in
//!   terms of which file types compiletest directives need to work on).
//!     - This refers only to the top-level metadata of a test file intended for compiletest, this
//!       does not include FileCheck directives or rustdoc test directives which are not intended
//!       for `compiletest`.
//!     - This is only applicable for the location-agonistic metadata, not ui test annotations like
//!       `//~ ERROR`.
//! - Each test belongs to a **test suite**, which in turn is based on a **test mode**.
//! - A **test mode** categorizes the foundational properties of how the tests under the specific
//!   mode is built and run, and are significantly distinct from other test modes.
//! - A **test suite** belongs to a test mode and is only different in how tests are built and run
//!   from other test suites under the same test mode in minor ways. For example, the `ui` test mode
//!   in `compiletest` currently consists of two test suites, `ui` and `ui-fulldeps`, the latter
//!   test suite is different in that the complete sysroot is made available to the tests.
//!
//! When adapted to this experiment:
//!
//! - A **directive** is the test metadata you write in tests as instructions and information for
//!   this library.
//! - Directives are collected, parsed then validated.
//! - Directives are transformed (should we call this lowered lol?) into **test properties**, which
//!   is the representation containing instructions and information that is suitable for consumption
//!   by test running implementations for each test mode and test suite.
//!     - This is intended to decouple the directive handling from what test running logic needs to
//!       handle.
//!     - Specifically, it is designed so that the test properties are strongly typed to prevent
//!       silly no-op directives or outright wrong instructions.
//!
//! # Design considerations in this experiment
//!
//! - We want directive handling to be robust.
//! - We want convenient error handling for someone who wants to introduce new directives or test
//!   modes/suites.
//! - We also want good diagnostics for test writers.
//!     - This includes unknown/redundant/invalid directives, unknown targets/arch/os/vendor,
//!       mutually exclusive directives, strange revisions (incl. trying to use a built-in cfg as
//!       revision), banned `compile-flags` etc.
//! - We want the design to remain open for tooling that want to operate on test metadata, e.g.
//!   collecting why tests are ignored.
//! - Considering spec and ferrocene cf. [test rule annotations], we probably want something like a
//!   `[references]` section that allows arbitrary keys (e.g. `fls` for ferrocene or `spec`), or
//!   make it easy to extend.
//!     - We should make it open and relatively easy for `compiletest` to report test metadata which
//!       can possibly be consumed by external tooling. E.g. `compiletest --report-test-metadata
//!       tests/ui/abi/compatibility.rs` might for example generate some JSON like (illustrative,
//!       format subject to change):
//!
//!       ```bash
//!       $ ./compiletest --report-test-metadata tests/ui/abi/compatibility.rs
//!       {
//!           "tests/ui/abi/compatibility.rs": {
//!               "revisions": ["foo", "bar"],
//!               "revision": {
//!                   "foo": {
//!                       "compile-flags": ["-C panic=abort"]
//!                   },
//!                   "bar": {
//!                       "cross-compile-targets": ["x86_64-pc-windows-msvc"]
//!                   },
//!               },
//!               "ignore": {
//!                   "target-vendors": [
//!                       { "vendor": "musl", "reason": "dylibs are not supported" }
//!                   ]
//!               },
//!               "mode": "check-fail",
//!               "compile-flags": ["-Zunstable-options"]
//!           }
//!       }
//!       ```
//!
//! [test rule annotations]:
//!     https://rust-lang.zulipchat.com/#narrow/stream/233931-t-compiler.2Fmajor-changes/topic/Test.20rule.20annotations.20compiler-team.23783
//!
//! # Remarks
//!
//! This structured directives experiment intentionally does not implement any kind of test
//! auxiliary logic since the core idea should still apply.
//!
//! This design considers the following cases:
//! - Directives can belong to either the "base" test, to specific revision(s), or to both.
//! - Some test modes do not support revisions.
//! - There's a distinction between test modes and test suites, where each test mode can correspond
//!   to multiple test suites.
//! - Some directives can be used only on the "base" test,
//! - Some directives are only valid in certain test modes.
//! - Some `compile-flags` are:
//!     - Invalid or banned under certain test modes (e.g. `-C incremental` under ui test mode).
//!     - Mutually exclusive with some other directives.
//! - First-class support for cross-compile ui/codegen/assembly/run-make tests, e.g.
//!   `cross-compile-targets` that replaces `compile-flags: --target thumbv8m.main-none-eabi` +
//!   `needs-llvm-components: arm` and catches something that tries to use both in combination
//!   instead of the dedicated directive.
//! - Some directives are mutually exclusive with each other.
//!
//! # Unresolved questions
//!
//! FIXME(jieyouxu): can this be made easy to extend? E.g. adding new directives and adding new test
//! modes / test suites.

/// A [`TestMode`] constitutes a common test running behavior and setup for its associated
/// [`TestSuite`]s.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TestMode {
    Ui,
    RunMake,
    Codegen,
    Assembly,
}

/// A [`TestSuite`] is a specialization of a [`TestMode`] which may introduce minor variations. If
/// the difference is significant enough, it should be made into its own [`TestMode`].
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TestSuite {
    Ui,
    UiFulldeps,
    RunMake,
    Codegen,
    Assembly,
}

impl TestSuite {
    /// Which [`TestMode`] this [`TestSuite`] belongs to.
    pub fn mode(self) -> TestMode {
        // Note: this is setup purposefully to demonstrate that a test mode is a one-to-many mapping
        // to test suites.
        match self {
            TestSuite::Ui | TestSuite::UiFulldeps => TestMode::Ui,
            TestSuite::RunMake => TestMode::RunMake,
            TestSuite::Codegen => TestMode::Codegen,
            TestSuite::Assembly => TestMode::Assembly,
        }
    }
}
