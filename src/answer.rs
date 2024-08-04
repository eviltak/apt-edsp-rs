use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Install {
    pub install: String,

    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Remove {
    pub remove: String,

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
pub enum Answer {
    Install(Install),
    Remove(Remove),
    Autoremove(Autoremove),
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
        let val = Answer::Install(Install {
            install: "abc".into(),
            ..Default::default()
        });
        assert_eq!(repr, rfc822_like::to_string(&val).unwrap());
    }
}
