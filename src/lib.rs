//! A Rust data model of the [APT External Dependency Solver Protocol][apt-edsp].
//! Useful for writing custom dependency solvers for [APT] in Rust.
//!
//! [apt-edsp]: https://salsa.debian.org/apt-team/apt/-/blob/a8367745eac915281cc2b9fb98813e9225d1e55c/doc/external-dependency-solver-protocol.md
//! [APT]: https://en.wikipedia.org/wiki/APT_(software)

pub use bool::Bool;
pub use progress::Progress;

#[cfg(test)]
mod test_util;

/// Contains the models for [EDSP answers].
///
/// [EDSP answers]: https://salsa.debian.org/apt-team/apt/-/blob/a8367745eac915281cc2b9fb98813e9225d1e55c/doc/external-dependency-solver-protocol.md#answer
pub mod answer;

/// Contains the models for the EDSP input (a [scenario]).
///
/// [scenario]: https://salsa.debian.org/apt-team/apt/-/blob/a8367745eac915281cc2b9fb98813e9225d1e55c/doc/external-dependency-solver-protocol.md#scenario
pub mod scenario;

mod bool;
mod progress;
mod util;
