use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::scenario::Version;

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Install {
    pub install: String,
    pub package: Option<String>,
    pub version: Option<Version>,
    pub architecture: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Remove {
    pub remove: String,
    pub package: Option<String>,
    pub version: Option<Version>,
    pub architecture: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Autoremove {
    pub autoremove: String,

    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Error {
    pub error: String,
    pub message: String,
}

#[derive(Serialize, Debug, Eq, PartialEq)]
#[serde(untagged)]
pub enum Action {
    Install(Install),
    Remove(Remove),
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

#[derive(Serialize, Debug, Eq, PartialEq)]
#[serde(untagged)]
pub enum Answer {
    Solution(Vec<Action>),
    Error(Error),
}

impl Answer {
    pub fn write_to(&self, writer: impl std::io::Write) -> Result<(), AnswerWriteError> {
        rfc822_like::to_writer(writer, self).map_err(Into::into)
    }
}

impl From<Error> for Answer {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

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
