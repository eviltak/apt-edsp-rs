use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::scenario::{Package, Version};

/// A stanza telling APT to install a specific new package, or to upgrade or downgrade a package
/// to a specific version.
#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Install {
    /// The identifier of the package to install.
    ///
    /// Must reference the identifier of a package in the package universe
    /// (see [`Package::id`]).
    pub install: String,

    /// The name of the package to install.
    ///
    /// While optional, it is highly recommend to set this field to the value of the field
    /// ([`Package::package`]) of the corresponding
    /// package in the package universe.
    pub package: Option<String>,

    /// The version of the package to install.
    ///
    /// While optional, it is highly recommend to set this field to the value of the field
    /// ([`Package::version`]) of the corresponding
    /// package in the package universe.
    pub version: Option<Version>,

    /// The architecture of the package to install.
    ///
    /// While optional, it is highly recommend to set this field to the value of the field
    /// ([`Package::architecture`]) of the corresponding
    /// package in the package universe.
    pub architecture: Option<String>,

    /// Extra optional fields supported by [`Package`] stanzas.
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

/// A stanza telling APT to remove a specific package.
#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Remove {
    /// The identifier of the package to remove.
    ///
    /// Must reference the identifier of a package in the package universe
    /// (see [`Package::id`]).
    pub remove: String,

    /// The name of the package to remove.
    ///
    /// While optional, it is highly recommend to set this field to the value of the field
    /// ([`Package::package`]) of the corresponding
    /// package in the package universe.
    pub package: Option<String>,

    /// The version of the package to remove.
    ///
    /// While optional, it is highly recommend to set this field to the value of the field
    /// ([`Package::version`]) of the corresponding
    /// package in the package universe.
    pub version: Option<Version>,

    /// The architecture of the package to remove.
    ///
    /// While optional, it is highly recommend to set this field to the value of the field
    /// ([`Package::architecture`]) of the corresponding
    /// package in the package universe.
    pub architecture: Option<String>,

    /// Extra optional fields supported by [`Package`] stanzas.
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

/// A stanza telling APT that a specific package can be autoremoved as a consequence of the
/// executed user request.
#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Autoremove {
    /// The identifier of the package that can be autoremoved.
    ///
    /// Must reference the identifier of a package in the package universe
    /// (see [`Package::id`]).
    pub autoremove: String,

    /// Extra optional fields supported by [`Package`] stanzas.
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

impl Package {
    /// Returns an [`Install`] stanza that can be used to tell APT to install this package.
    pub fn to_install(&self) -> Install {
        Install {
            install: self.id.clone(),
            package: Some(self.package.clone()),
            version: Some(self.version.clone()),
            architecture: Some(self.architecture.clone()),
            ..Default::default()
        }
    }

    /// Returns a [`Remove`] stanza that can be used to tell APT to remove this package.
    pub fn to_remove(&self) -> Remove {
        Remove {
            remove: self.id.clone(),
            package: Some(self.package.clone()),
            version: Some(self.version.clone()),
            architecture: Some(self.architecture.clone()),
            ..Default::default()
        }
    }

    /// Returns an [`Autoremove`] stanza that can be used to tell APT that this package can be
    /// autoremoved.
    pub fn to_autoremove(&self) -> Autoremove {
        Autoremove {
            autoremove: self.id.clone(),
            ..Default::default()
        }
    }
}

/// An [Error stanza][error] reporting the error(s) faced when trying to fulfill an
/// unsatisfiable user request.
///
/// [error]: https://salsa.debian.org/apt-team/apt/-/blob/a8367745/doc/external-dependency-solver-protocol.md#error
#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Error {
    /// A unique error identifier, such as a UUID. The value of this field is ignored.
    pub error: String,

    /// Human-readable text explaining the cause of the solver error.
    ///
    /// If multiline, the first line conveys a short message, which is then explained in more
    /// detail in subsequent lines.
    pub message: String,
}

/// A stanza in an [`Answer::Solution`].
#[derive(Serialize, Debug, Eq, PartialEq)]
#[serde(untagged)]
pub enum Action {
    /// A single [`Install`] stanza in an [`Answer::Solution`].
    Install(Install),
    /// A single [`Remove`] stanza in an [`Answer::Solution`].
    Remove(Remove),
    /// A single [`Autoremove`] stanza in an [`Answer::Solution`].
    Autoremove(Autoremove),
}

impl From<Install> for Action {
    fn from(value: Install) -> Self {
        Self::Install(value)
    }
}

impl From<Remove> for Action {
    fn from(value: Remove) -> Self {
        Self::Remove(value)
    }
}

impl From<Autoremove> for Action {
    fn from(value: Autoremove) -> Self {
        Self::Autoremove(value)
    }
}

/// The [answer] returned from the external solver to APT upon completion of the dependency
/// resolution process.
///
/// [answer]: https://salsa.debian.org/apt-team/apt/-/blob/a8367745/doc/external-dependency-solver-protocol.md#answer
#[derive(Serialize, Debug, Eq, PartialEq)]
#[serde(untagged)]
pub enum Answer {
    /// A list of stanzas describing the [`Action`]s to be made to the set of installed packages
    /// to satisfy the user's request.
    Solution(Vec<Action>),
    /// A single [`Error`] stanza reporting an error during the dependency resolution process.
    Error(Error),
}

impl Answer {
    /// Writes this [`Answer`] to the given `writer`. On error, returns an [`AnswerWriteError`].
    pub fn write_to(&self, writer: impl std::io::Write) -> Result<(), AnswerWriteError> {
        rfc822_like::to_writer(writer, self).map_err(Into::into)
    }
}

impl From<Error> for Answer {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

/// The error returned when [`Answer::write_to`] fails.
///
/// Though the implementation details are hidden, the struct implements [`std::error::Error`]
/// and a human-friendly [`std::fmt::Display`] implementation.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct AnswerWriteError(#[from] rfc822_like::ser::Error);

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::test_util::ser_test;

    use super::*;

    ser_test! {
        test_action: {
            indoc! {"
                Install: abc
            "} =>
            Action::Install(Install {
                install: "abc".into(),
                ..Default::default()
            }),
        }
    }

    ser_test! {
        test_answer: {
            indoc! {"
                Install: 123
                Architecture: amd64

                Remove: 234
                Package: bar
                Version: 0.1.2

                Autoremove: 345
            "} =>
            Answer::Solution(
                vec![
                    Install {
                        install: "123".into(),
                        architecture: Some("amd64".into()),
                        ..Default::default()
                    }.into(),
                    Remove {
                        remove: "234".into(),
                        package: Some("bar".into()),
                        version: Some("0.1.2".try_into().unwrap()),
                        ..Default::default()
                    }.into(),
                    Autoremove {
                        autoremove: "345".into(),
                        ..Default::default()
                    }.into(),
                ]
            ),
            indoc! {"
                Error: foo
                Message: bar
                 baz
            "} =>
            Answer::Error(Error {
                error: "foo".to_string(),
                message: "bar\nbaz".to_string(),
            }),
        }
    }
}
