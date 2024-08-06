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

/// Describes an [APT EDSP scenario][scenario].
///
/// [scenario]: https://salsa.debian.org/apt-team/apt/-/blob/a8367745/doc/external-dependency-solver-protocol.md#scenario
pub struct Scenario {
    /// The [`Request`] stanza.
    pub request: Request,

    /// The [`Package`] stanzas comprising the [package universe][universe].
    ///
    /// [universe]: https://salsa.debian.org/apt-team/apt/-/blob/a8367745/doc/external-dependency-solver-protocol.md#package-universe
    pub universe: Vec<Package>,
}

impl Scenario {
    /// Reads a [`Scenario`] from the given `reader`. On error, returns an [`ScenarioReadError`].
    pub fn read_from(mut reader: impl BufRead) -> Result<Self, ScenarioReadError> {
        let request: Request = rfc822_like::from_reader(&mut reader)?;
        let universe: Vec<Package> = rfc822_like::from_reader(&mut reader)?;
        Ok(Scenario { request, universe })
    }
}

/// The error returned when [`Scenario::read_from`] fails.
///
/// Though the implementation details are hidden, the struct implements [`std::error::Error`]
/// and a human-friendly [`std::fmt::Display`] implementation.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct ScenarioReadError(#[from] rfc822_like::de::Error);

/// Encapsulates the _action_ fields in a [`Request`] stanza.
#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Actions {
    /// (deprecated) Set to [`Bool::YES`] in an APT `dist-upgrade` request. Defaults to
    /// [`Bool::NO`].
    ///
    /// Equivalent to setting [`Actions::upgrade_all`] to [`Bool::YES`], and
    /// [`Preferences::forbid_new_install`] and [`Preferences::forbid_remove`] to [`Bool::NO`].
    #[serde(rename = "Dist-Upgrade")]
    pub dist_upgrade: Bool,

    /// (deprecated) Set to [`Bool::YES`] in an APT `upgrade` request. Defaults to [`Bool::NO`].
    ///
    /// Equivalent to setting [`Actions::upgrade_all`], [`Preferences::forbid_new_install`] and
    /// [`Preferences::forbid_remove`] to [`Bool::YES`].
    pub upgrade: Bool,

    /// If set to [`Bool::YES`], a cleanup of unused automatically installed packages has been
    /// requested, usually via an APT `autoremove` request. Defaults to [`Bool::NO`].
    pub autoremove: Bool,

    /// If set to [`Bool::YES`], an upgrade of all installed packages has been requested,
    /// usually via an upgrade command like `apt full-upgrade`. Defaults to [`Bool::NO`].
    #[serde(rename = "Upgrade-All")]
    pub upgrade_all: Bool,

    /// A space-separated list of arch-qualified package names, with no version attached, to
    /// remove. A value of [`None`] denotes an empty list.
    pub remove: Option<String>,

    /// A space-separated list of arch-qualified package names, with no version attached, to
    /// install. A value of [`None`] denotes an empty list.
    pub install: Option<String>,
}

/// Encapsulates the _preference_ fields in a [`Request`] stanza.
#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Preferences {
    /// When set to [`Bool::YES`], APT pinning is _strict_, i.e. the solver must not propose to
    /// install packages which are not APT candidates[^note]. When set to [`Bool::NO`], the solver
    /// does only a best effort attempt to install APT candidates. Defaults to [`Bool::YES`].
    ///
    /// [^note]: See [`Package::pin`] and [`Package::candidate`].
    #[serde(rename = "Strict-Pinning")]
    pub strict_pinning: Bool<true>,

    /// When set to [`Bool::YES`] the resolver is forbidden to install new packages in its
    /// returned solution. Defaults to [`Bool::NO`].
    #[serde(rename = "Forbid-New-Install")]
    pub forbid_new_install: Bool,

    /// When set to [`Bool::YES`] the resolver is forbidden to remove currently installed
    /// packages in its returned solution. Defaults to [`Bool::NO`].
    #[serde(rename = "Forbid-Remove")]
    pub forbid_remove: Bool,

    /// A purely informational string specifying the solver to which this request was initially
    /// sent.
    pub solver: Option<String>,

    /// A solver-specific preferences string, usually coming from the `APT::Solver::Preferences`
    /// APT configuration option.
    pub preferences: Option<String>,
}

/// The [request stanza][req] of a [`Scenario`].
///
/// [req]: https://salsa.debian.org/apt-team/apt/-/blob/a8367745/doc/external-dependency-solver-protocol.md#request
#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Request {
    /// The EDSP protocol used to communicate with APT.
    pub request: String,

    /// The name of the native architecture on the user machine.
    pub architecture: String,

    /// A space separated list of all architectures known to APT.
    pub architectures: Option<String>,

    /// The action fields in a [`Request`] stanza.
    #[serde(flatten)]
    pub actions: Actions,

    /// The preference fields in a [`Request`] stanza.
    #[serde(flatten)]
    pub preferences: Preferences,
}

/// Describes an installed or available package in the [package universe][universe].
///
/// [universe]: https://salsa.debian.org/apt-team/apt/-/blob/a8367745/doc/external-dependency-solver-protocol.md#package-universe
#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Package {
    /// The name of the package.
    pub package: String,

    /// The version of the package.
    pub version: Version,

    /// A string representing the [architecture(s)][arch] the package supports.
    ///
    /// [arch]: https://www.debian.org/doc/debian-policy/ch-controlfields.html#architecture
    pub architecture: String,

    /// If set to [`Bool::YES`], the package is installed in the system. Defaults to [`Bool::NO`].
    pub installed: Bool,

    /// If set to [`Bool::YES`], the package is marked as "on hold" by `dpkg`. Defaults to
    /// [`Bool::NO`].
    pub hold: Bool,

    /// The unique package identifier, according to APT.
    #[serde(rename = "APT-ID")]
    pub id: String,

    /// The package pin value, according to APT policy.
    #[serde(rename = "APT-Pin", with = "super::util::serde_as_string")]
    pub pin: u32,

    /// If set to [`Bool::YES`], the package is the APT candidate for installation among all
    /// available packages with the same name and architecture. Defaults to [`Bool::NO`].
    #[serde(rename = "APT-Candidate")]
    pub candidate: Bool,

    /// If set to [`Bool::YES`], the package is marked by APT as automatic installed.
    #[serde(rename = "APT-Automatic")]
    pub automatic: Bool,

    /// Specifies the absolute dependencies of the package. See the [Debian Policy Manual][man]
    /// on the `Depends` field for more information.
    ///
    /// [man]: https://www.debian.org/doc/debian-policy/ch-relationships.html#binary-dependencies-depends-recommends-suggests-enhances-pre-depends
    #[serde(default)]
    pub depends: Vec<Dependency>,

    /// Specifies packages that conflict with this package. See the [Debian Policy Manual][man]
    /// on the `Conflicts` field for more information.
    ///
    /// [man]: https://www.debian.org/doc/debian-policy/ch-relationships.html#conflicting-binary-packages-conflicts
    #[serde(default)]
    pub conflicts: Vec<Relationship>,

    /// Contains other optional fields that can be contained in a [`Package`] stanza.
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}
