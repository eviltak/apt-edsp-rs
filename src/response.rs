use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Install {
    pub install: String,

    #[serde(flatten)]
    extra: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Remove {
    pub remove: String,

    #[serde(flatten)]
    extra: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Autoremove {
    autoremove: String,

    #[serde(flatten)]
    extra: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Error {
    error: String,
    message: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Progress {
    progress: String,
    percentage: Option<String>,
    message: Option<String>,
}

// Deserialize implemented manually below
#[derive(Serialize, Debug, Eq, PartialEq)]
#[serde(untagged)]
pub enum Response {
    Install(Install),
    Remove(Remove),
    Autoremove(Autoremove),
    Error(Error),
    Progress(Progress),
    Empty,
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
        let val = Response::Install(Install { install: "abc".into(), ..Default::default() });
        assert_eq!(repr, rfc822_like::to_string(&val).unwrap());
    }
}
