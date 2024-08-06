use std::collections::HashMap;
use std::io::BufRead;

use serde::{Deserialize, Serialize};

pub use relationship::{
    Dependency, DependencyParseError, Relation, Relationship, RelationshipParseError,
};
pub use version::Version;

use super::Bool;

mod relationship;
mod version;

#[cfg(test)]
mod tests;

pub struct Scenario {
    pub request: Request,
    pub universe: Vec<Package>,
}

impl Scenario {
    pub fn read_from(mut reader: impl BufRead) -> Result<Self, ScenarioReadError> {
        let request: Request = rfc822_like::from_reader(&mut reader)?;
        let universe: Vec<Package> = rfc822_like::from_reader(&mut reader)?;
        Ok(Scenario { request, universe })
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct ScenarioReadError(#[from] rfc822_like::de::Error);

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Actions {
    #[serde(rename = "Dist-Upgrade")]
    pub dist_upgrade: Bool,
    pub upgrade: Bool,
    pub autoremove: Bool,
    #[serde(rename = "Upgrade-All")]
    pub upgrade_all: Bool,
    pub remove: Option<String>,
    pub install: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Preferences {
    #[serde(rename = "Strict-Pinning")]
    pub strict_pinning: Bool<true>,
    #[serde(rename = "Forbid-New-Install")]
    pub forbid_new_install: Bool,
    #[serde(rename = "Forbid-Remove")]
    pub forbid_remove: Bool,
    pub solver: Option<String>,
    pub preferences: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Request {
    pub request: String,
    pub architecture: String,
    pub architectures: Option<String>,
    #[serde(flatten)]
    pub actions: Actions,
    #[serde(flatten)]
    pub preferences: Preferences,
}

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Package {
    pub package: String,
    pub version: Version,
    pub architecture: String,
    pub installed: Bool,
    pub hold: Bool,
    #[serde(rename = "APT-ID")]
    pub id: String,
    #[serde(rename = "APT-Pin", with = "super::util::serde_as_string")]
    pub pin: u32,
    #[serde(rename = "APT-Candidate")]
    pub candidate: Bool,
    #[serde(rename = "APT-Automatic")]
    pub automatic: Bool,
    #[serde(default)]
    pub depends: Vec<Dependency>,
    #[serde(default)]
    pub conflicts: Vec<Relationship>,

    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}
