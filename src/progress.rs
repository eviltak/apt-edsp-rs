use serde::{Deserialize, Serialize};

/// The model describing a [`Progress` stanza].
///
/// [`Progress` stanza]: https://salsa.debian.org/apt-team/apt/-/blob/a8367745eac915281cc2b9fb98813e9225d1e55c/doc/external-dependency-solver-protocol.md#progress
#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Progress {
    pub progress: String,
    pub percentage: Option<String>,
    pub message: Option<String>,
}

impl Progress {
    pub fn write_to(&self, writer: impl std::io::Write) -> Result<(), ProgressWriteError> {
        rfc822_like::to_writer(writer, self).map_err(Into::into)
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct ProgressWriteError(#[from] rfc822_like::ser::Error);
