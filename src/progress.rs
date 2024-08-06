use serde::{Deserialize, Serialize};

/// The model describing a [`Progress` stanza].
///
/// [`Progress` stanza]: https://salsa.debian.org/apt-team/apt/-/blob/a8367745eac915281cc2b9fb98813e9225d1e55c/doc/external-dependency-solver-protocol.md#progress
#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Progress {
    /// Must contain a date and time timestamp from the UTC timezone, in RFC 2822 format.
    pub progress: String,

    /// An integer from 0 to 100, representing the completion of the dependency solving process,
    /// as declared by the solver.
    pub percentage: Option<String>,

    /// A textual message, meant to be read by the APT user, describing what is going on
    /// within the dependency solving process (e.g. the current phase of dependency solving,
    /// as declared by the solver).
    pub message: Option<String>,
}

impl Progress {
    /// Writes this [`Progress`] to the given `writer`. On error, returns an [`ProgressWriteError`].
    pub fn write_to(&self, writer: impl std::io::Write) -> Result<(), ProgressWriteError> {
        rfc822_like::to_writer(writer, self).map_err(Into::into)
    }
}

/// The error returned when [`Progress::write_to`] fails.
///
/// Though the implementation details are hidden, the struct implements [`std::error::Error`]
/// and a human-friendly [`std::fmt::Display`] implementation.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct ProgressWriteError(#[from] rfc822_like::ser::Error);
