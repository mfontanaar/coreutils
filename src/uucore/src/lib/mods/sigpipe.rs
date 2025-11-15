// This file is part of the uutils coreutils package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(target_family = "unix")]
use libc::{SIG_DFL, SIGPIPE, sighandler_t, signal};

pub fn fix_sigpipe_handling() -> sighandler_t {
    // By default Rust doesn't handle SIGPIPE, it ignores it:
    // https://github.com/rust-lang/rust/issues/62569
    // This is inconvenient for POSIX compliance, as it is required that
    // the default is SIGPIPE -> SIG_DFL (except for certain cases).
    // We expose this at the top level of `uucore` so that procedural macros like
    // `main!` can use it to fix SIGPIPE handling without requiring extra dependencies
    // or features.
    // If https://github.com/rust-lang/rust/issues/97889 ever gets stabilized,
    // this whole logic can be removed and the consuming macros simplified.

    // Code for the same intent exists under the `signals` feature. However,
    // that feature is not enabled by default, and also should be kept even if
    // this file (and module) gets removed, as it is meant for utilities dealing
    // with fine-grained control over signal handling.
    unsafe { signal(SIGPIPE, SIG_DFL) }
}
