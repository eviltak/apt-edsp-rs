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

pub enum Answer {
    Solution(Vec<Action>),
    Error(Error),
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn test_answer() {
        let repr = indoc! {"
            Install: abc
        "};
        let val = Action::Install(Install {
            install: "abc".into(),
            ..Default::default()
        });
        assert_eq!(repr, rfc822_like::to_string(&val).unwrap());
    }
}
