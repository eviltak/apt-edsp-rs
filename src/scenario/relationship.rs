use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::super::util::TryFromStringVisitor;
use super::Version;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Relation {
    Earlier,
    EarlierEqual,
    Equal,
    LaterEqual,
    Later,
}

impl Relation {
    pub fn parse<'a, E: nom::error::ParseError<&'a str>>(
        input: &'a str,
    ) -> nom::IResult<&str, Self, E> {
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
            RelationshipParseError::EmptyPackageName(e) => {
                write!(f, "Error parsing package name:\n{e}")
            }
            RelationshipParseError::BadConstraintSpec(e) => {
                write!(f, "Error parsing constraint spec:\n{e}")
            }
            RelationshipParseError::BadVersion(e) => write!(f, "Error parsing version:\n{e}"),
        }
    }
}

impl FromStr for Relationship {
    type Err = RelationshipParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        use nom::bytes::complete::*;
        use nom::character::complete::*;
        use nom::combinator::*;
        use nom::error::{context, convert_error};
        use nom::sequence::*;
        use nom::Finish;

        let (remaining, package) = terminated(
            context(
                "package name",
                take_while1(|c: char| !c.is_whitespace() && c != '('),
            ),
            space0,
        )(input)
        .finish()
        .map_err(|e| RelationshipParseError::EmptyPackageName(convert_error(input, e)))?;

        // Parse constraint
        let constraint = if remaining.is_empty() {
            None
        } else {
            let (_, (relation, version)) = all_consuming(context(
                "spec",
                preceded(
                    char('('),
                    terminated(
                        separated_pair(
                            context("relation", Relation::parse),
                            space0,
                            context("version", take_until1(")")),
                        ),
                        tuple((char(')'), space0)),
                    ),
                ),
            ))(remaining)
            .finish()
            .map_err(|e| RelationshipParseError::BadConstraintSpec(convert_error(input, e)))?;
            let version = Version::try_from(version).map_err(RelationshipParseError::BadVersion)?;
            Some((relation, version))
        };

        Ok(Self {
            package: package.to_string(),
            constraint,
        })
    }
}

impl TryFrom<String> for Relationship {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl TryFrom<&str> for Relationship {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl Serialize for Relationship {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Relationship {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(TryFromStringVisitor::new())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Dependency {
    pub first: Relationship,
    pub alternates: Vec<Relationship>,
}

impl Display for Dependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.first)?;

        for alt in &self.alternates {
            write!(f, " | {}", alt)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum DependencyParseError {
    Alternate(usize, RelationshipParseError),
}

impl Display for DependencyParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyParseError::Alternate(i, e) => write!(f, "Error parsing alternate {i}: {e}"),
        }
    }
}

impl FromStr for Dependency {
    type Err = DependencyParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (first, rest) = input.split_once('|').unwrap_or((input, ""));

        let first = first
            .trim()
            .parse()
            .map_err(|e| DependencyParseError::Alternate(0, e))?;
        let alternates = if !rest.is_empty() {
            rest.split('|')
                .map(|s| s.trim())
                .enumerate()
                .map(|(i, s)| {
                    s.parse()
                        .map_err(|e| DependencyParseError::Alternate(i + 1, e))
                })
                .collect::<Result<_, _>>()?
        } else {
            vec![]
        };

        Ok(Self { first, alternates })
    }
}

impl TryFrom<String> for Dependency {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl TryFrom<&str> for Dependency {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl Serialize for Dependency {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Dependency {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(TryFromStringVisitor::new())
    }
}
