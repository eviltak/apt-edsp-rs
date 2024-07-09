use std::collections::HashMap;
use std::io::BufRead;

use serde::{Deserialize, Serialize};

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
    pub version: String,
    pub architecture: String,
    #[serde(rename = "APT-ID")]
    pub id: String,
    #[serde(rename = "APT-Pin")]
    pub pin: String,
    #[serde(rename = "APT-Candidate")]
    pub candidate: Option<String>,
    pub depends: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    struct TestCase<T> {
        repr: &'static str,
        val: T,
    }

    impl<T: Eq + Serialize + Deserialize<'static> + std::fmt::Debug> TestCase<T> {
        fn check(&self) {
            assert_eq!(self.repr, rfc822_like::to_string(&self.val).unwrap());
            assert_eq!(self.val, rfc822_like::from_str(self.repr).unwrap());
        }
    }

    macro_rules! tests {
        ($($name:ident: {$repr:expr, $val:expr})*) => {
            $(
                #[test]
                fn $name() {
                    let repr = { $repr };
                    let val = { $val };
                    TestCase { repr, val, }.check();
                }
            )*
        };
    }

    tests! {
        request: {
            indoc! {"
                Request: EDSP 0.5
                Architecture: amd64
                Upgrade-All: yes
            "},
            Request {
                request: "EDSP 0.5".into(),
                architecture: "amd64".into(),
                actions: Actions {
                    upgrade_all: Some("yes".into()),
                    ..Default::default()
                },
                ..Default::default()
            }
        }
        vec_request: {
            indoc! {"
                Request: EDSP 0.5
                Architecture: amd64
                Upgrade-All: yes

                Request: EDSP 0.5
                Architecture: amd64
                Upgrade-All: no
            "},
            vec![
                Request {
                    request: "EDSP 0.5".into(),
                    architecture: "amd64".into(),
                    actions: Actions {
                        upgrade_all: Some("yes".into()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Request {
                    request: "EDSP 0.5".into(),
                    architecture: "amd64".into(),
                    actions: Actions {
                        upgrade_all: Some("no".into()),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            ]
        }
    }
}
