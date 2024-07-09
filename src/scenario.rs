use std::collections::HashMap;
use std::fmt::Display;
use std::io::BufRead;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::Version;

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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Relation {
    Earlier,
    EarlierEqual,
    Equal,
    LaterEqual,
    Later,
}

impl Relation {
    pub fn parse<'a, E: nom::error::ParseError<&'a str>>(input: &'a str) -> nom::IResult<&str, Self, E> {
        use nom::branch::alt;
        use nom::bytes::complete::tag;
        use nom::combinator::value;
        alt((
            value(Relation::Earlier, tag("<<")),
            value(Relation::EarlierEqual, tag("<=")),
            value(Relation::Equal, tag("=")),
            value(Relation::LaterEqual, tag(">=")),
            value(Relation::Later, tag(">>")),
        ))(input)
    }
}

impl Display for Relation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Relation::Earlier => write!(f, "<<"),
            Relation::EarlierEqual => write!(f, "<="),
            Relation::Equal => write!(f, "="),
            Relation::LaterEqual => write!(f, ">="),
            Relation::Later => write!(f, ">>"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Relationship {
    pub package: String,
    pub constraint: Option<(Relation, Version)>,
}

impl Display for Relationship {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.package)?;

        if let Some((relation, version)) = &self.constraint {
            write!(f, " ({} {})", relation, version.as_str())?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum RelationshipParseError {
    EmptyPackageName(String),
    BadConstraintSpec(String),
    BadVersion(<Version as TryFrom<&'static str>>::Error),
}

impl Display for RelationshipParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationshipParseError::EmptyPackageName(e) => write!(f, "Error parsing package name: {}", e),
            RelationshipParseError::BadConstraintSpec(e) => write!(f, "Error parsing constraint spec: {}", e),
            RelationshipParseError::BadVersion(e) => write!(f, "Error parsing version: {}", e)
        }
    }
}

impl FromStr for Relationship {
    type Err = RelationshipParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        use nom::Finish;
        use nom::bytes::complete::*;
        use nom::character::complete::*;
        use nom::combinator::*;
        use nom::sequence::*;

        let (remaining, package) = terminated(
            take_while1(|c: char| !c.is_whitespace() && c != '('),
            space0,
        )(input)
            .finish()
            .map_err(|e| RelationshipParseError::EmptyPackageName(nom::error::convert_error(input, e)))?;

        // Parse constraint
        let constraint = if remaining.is_empty() {
            None
        } else {
            let (_, (relation, version)) = all_consuming(preceded(
                char('('),
                terminated(
                    separated_pair(Relation::parse, space0, take_until1(")")),
                    tuple((char(')'), space0)),
                ),
            ))(remaining)
                .finish()
                .map_err(|e| RelationshipParseError::BadConstraintSpec(nom::error::convert_error(input, e)))?;
            let version = Version::try_from(version).map_err(RelationshipParseError::BadVersion)?;
            Some((relation, version))
        };

        Ok(Self {
            package: package.to_string(),
            constraint,
        })
    }
}

impl Serialize for Relationship {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Relationship {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)
            .and_then(|v| v.parse().map_err(serde::de::Error::custom))
    }
}


#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    struct TestCase<T> {
        repr: &'static str,
        val: T,
    }

    impl<T: Eq + Serialize + Deserialize<'static> + std::fmt::Debug> TestCase<T>
    {
        fn check<FS, FD>(&self, serialize_fn: FS, deserialize_fn: FD)
        where
            FS: Fn(&T) -> String,
            FD: Fn(&'static str) -> T,
        {
            assert_eq!(self.val, (deserialize_fn)(self.repr), "Incorrect deserialized value from '{}' (left: expected, right: actual)", self.repr);
            assert_eq!(self.repr, (serialize_fn)(&self.val), "Incorrect serialized value from '{:?}' (left: expected, right: actual)", self.val);
        }
    }

    fn struct_to_string<T: Serialize>(val: &T) -> String {
        match rfc822_like::to_string(val) {
            Ok(t) => t,
            Err(e) => panic!("Error when serializing: {e}"),
        }
    }

    fn struct_from_str<'de, T: Deserialize<'de>>(s: &'de str) -> T {
        match rfc822_like::from_str(s) {
            Ok(t) => t,
            Err(e) => panic!("Error when deserializing: {e}"),
        }
    }

    fn value_to_string<T: Serialize>(val: &T) -> String {
        #[derive(Serialize)]
        struct Record<'a, V> {
            val: &'a V,
        }

        struct_to_string(&Record { val }).trim()["val: ".len()..].to_string()
    }

    fn value_from_str<T: for<'de> Deserialize<'de>>(s: &str) -> T {
        #[derive(Deserialize)]
        struct Record<V> {
            val: V,
        }

        struct_from_str::<Record<T>>(&format!("val: {s}")).val
    }

    macro_rules! serde_test {
        ($name:ident: {$($repr:expr => $val:expr),+}) => {
            serde_test!(
                $name(
                struct_to_string,
                struct_from_str
                ): {$($repr => $val),+}
            );
        };

        ($name:ident($serialize_fn:expr, $deserialize_fn:expr): {$($repr:expr => $val:expr),+}) => {
            serde_test!(@test
                $name,
                $serialize_fn,
                $deserialize_fn, $($repr, $val)+;
            );
        };

        (@test $name:ident, $serialize_fn:expr, $deserialize_fn:expr, $($repr:expr, $val:expr)+;) => {
            #[test]
            fn $name() {
                $(
                {
                    let repr = { $repr };
                    let val = { $val };
                    TestCase {
                        repr,
                        val,
                    }.check($serialize_fn, $deserialize_fn);
                }
                )+
            }
        };
    }

    serde_test! {
        request: {
            indoc! {"
                Request: EDSP 0.5
                Architecture: amd64
                Upgrade-All: yes
            "} =>
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
    }

    serde_test! {
        vec_request: {
            indoc! {"
                Request: EDSP 0.5
                Architecture: amd64
                Upgrade-All: yes

                Request: EDSP 0.5
                Architecture: amd64
                Upgrade-All: no
            "} =>
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

    serde_test! {
        relationship(value_to_string, value_from_str): {
            "foo" =>
            Relationship {
                package: "foo".into(),
                constraint: None,
            },
            "foo (<< 2.2.1)" =>
            Relationship {
                package: "foo".into(),
                constraint: Some((Relation::Earlier, Version::try_from("2.2.1").unwrap())),
            },
            "foo (<= 2.2.1)" =>
            Relationship {
                package: "foo".into(),
                constraint: Some((Relation::EarlierEqual, Version::try_from("2.2.1").unwrap())),
            },
            "foo (= 2.2.1)" =>
            Relationship {
                package: "foo".into(),
                constraint: Some((Relation::Equal, Version::try_from("2.2.1").unwrap())),
            },
            "foo (>= 2.2.1)" =>
            Relationship {
                package: "foo".into(),
                constraint: Some((Relation::LaterEqual, Version::try_from("2.2.1").unwrap())),
            },
            "foo (>> 2.2.1)" =>
            Relationship {
                package: "foo".into(),
                constraint: Some((Relation::Later, Version::try_from("2.2.1").unwrap())),
            }
        }
    }

    serde_test! {
        vec_relationship(value_to_string, value_from_str): {
            indoc! {"
                foo,
                     bar,
                     baz
            "}.trim() =>
            vec![
                Relationship {
                    package: "foo".into(),
                    constraint: None,
                },
                Relationship {
                    package: "bar".into(),
                    constraint: None,
                },
                Relationship {
                    package: "baz".into(),
                    constraint: None,
                }
            ]
        }
    }
}
