use std::collections::HashMap;
use std::fmt::Display;
use std::io::BufRead;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub use relationship::{Dependency, DependencyParseError, Relation, Relationship, RelationshipParseError};

use super::Version;

mod relationship;

#[cfg(test)]
mod tests;

pub struct Scenario {
    pub request: Request,
    pub universe: Vec<Package>,
}

impl Scenario {
    pub fn from_read<R: BufRead>(mut reader: R) -> Self {
        log::info!("Parsing scenario...");

        let request: Request = rfc822_like::from_reader(&mut reader).unwrap();

        log::debug!("Parsed request: {:#?}", request);

        let universe: Vec<Package> = rfc822_like::from_reader(&mut reader).unwrap();

        log::debug!("Parsed universe with {} packages", universe.len());

        Scenario {
            request,
            universe,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Actions {
    #[serde(rename = "Dist-Upgrade")]
    pub dist_upgrade: Option<String>,
    pub upgrade: Option<String>,
    pub autoremove: Option<String>,
    #[serde(rename = "Upgrade-All")]
    pub upgrade_all: Option<String>,
    pub remove: Option<String>,
    pub install: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Preferences {
    #[serde(rename = "Strict-Pinning")]
    pub strict_pinning: Option<String>,
    pub solver: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
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
    pub installed: Option<String>,
    #[serde(rename = "APT-ID")]
    pub id: String,
    #[serde(rename = "APT-Pin")]
    pub pin: String,
    #[serde(rename = "APT-Candidate")]
    pub candidate: Option<String>,
    pub depends: Vec<Dependency>,
    pub conflicts: Vec<Relationship>,

    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}
