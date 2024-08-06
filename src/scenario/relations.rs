use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::super::util::TryFromStringVisitor;
use super::Version;

/// Specifies the comparator used to compare two [`Version`]s.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Relation {
    /// The first version must be strictly earlier than the second (`a < b`).
    Earlier,
    /// The first version must be earlier than or equal to the second (`a <= b`).
    EarlierEqual,
    /// The first version must be equal to the second (`a == b`).
    Equal,
    /// The first version must be later than or equal to the second (`a >= b`).
    LaterEqual,
    /// The first version must be strictly later than the second (`a > b`).
    Later,
}

impl Relation {
    fn parse<'a, E: nom::error::ParseError<&'a str>>(
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

/// Describes a set of versions of a package.
#[derive(Debug, Eq, PartialEq)]
pub struct VersionSet {
    /// The name of the package.
    pub package: String,
    /// The constraint fulfilled by the versions in the version set. If [`None`], the version set
    /// contains _all_ the versions of the given package.
    pub constraint: Option<(Relation, Version)>,
}

impl Display for VersionSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.package)?;

        if let Some((relation, version)) = &self.constraint {
            write!(f, " ({} {})", relation, version.as_str())?;
        }

        Ok(())
    }
}

/// The error returned when failing to parse a [`VersionSet`].
#[derive(Debug)]
pub enum VersionSetParseError {
    /// The package name was empty. Contains the trace information for where the error was found.
    EmptyPackageName(String),

    /// There was an error parsing the constraint. Contains the trace information for where
    /// the error was found.
    BadConstraintSpec(String),

    /// There was an error parsing the [`Version`].
    BadVersion(<Version as TryFrom<&'static str>>::Error),
}

impl Display for VersionSetParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionSetParseError::EmptyPackageName(e) => {
                write!(f, "Error parsing package name:\n{e}")
            }
            VersionSetParseError::BadConstraintSpec(e) => {
                write!(f, "Error parsing constraint spec:\n{e}")
            }
            VersionSetParseError::BadVersion(e) => write!(f, "Error parsing version:\n{e}"),
        }
    }
}

impl FromStr for VersionSet {
    type Err = VersionSetParseError;

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
        .map_err(|e| VersionSetParseError::EmptyPackageName(convert_error(input, e)))?;

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
            .map_err(|e| VersionSetParseError::BadConstraintSpec(convert_error(input, e)))?;
            let version = Version::try_from(version).map_err(VersionSetParseError::BadVersion)?;
            Some((relation, version))
        };

        Ok(Self {
            package: package.to_string(),
            constraint,
        })
    }
}

impl TryFrom<String> for VersionSet {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl TryFrom<&str> for VersionSet {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl Serialize for VersionSet {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for VersionSet {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(TryFromStringVisitor::new())
    }
}

/// Specifies a dependency of a package that can be fulfilled by one or more [`VersionSet`]s.
#[derive(Debug, Eq, PartialEq)]
pub struct Dependency {
    /// The first [`VersionSet`] that can fulfill this [`Dependency`].
    pub first: VersionSet,
    /// The other [`VersionSet`]s that can fulfill this [`Dependency`].
    pub alternates: Vec<VersionSet>,
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

/// The error returned when failing to parse a [`Dependency`].
#[derive(Debug)]
pub enum DependencyParseError {
    /// There was an error parsing the [`VersionSet`] with the given index.
    Alternate(usize, VersionSetParseError),
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
