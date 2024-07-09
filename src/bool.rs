use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::util::TryFromStringVisitor;

/// A [`bool`] wrapper type that serializes `true` and `false` to `"yes"` and `"no"` respectively.
///
/// # Examples
/// ```
/// # use apt_edsp::Bool;
///
/// assert_eq!("yes", Bool::YES.as_str());
/// assert_eq!(Bool::NO, "no".parse().unwrap());
/// ```
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Bool(pub bool);

impl Bool {
    /// Equivalent to `true` or `"yes"`.
    pub const YES: Self = Bool(true);

    /// Equivalent to `false` or `"no`".
    pub const NO: Self = Bool(false);

    /// Returns the [`Bool`] corresponding to `"yes"`.
    pub fn yes() -> Self {
        Self::YES
    }

    /// Returns the [`Bool`] corresponding to `"no"`.
    pub fn no() -> Self {
        Self::NO
    }

    /// Returns the string literal representation (`"yes"` or `"no"`) for this boolean value.
    pub fn as_str(&self) -> &'static str {
        if self.0 {
            "yes"
        } else {
            "no"
        }
    }
}

impl From<Bool> for &'static str {
    fn from(value: Bool) -> Self {
        value.as_str()
    }
}

impl FromStr for Bool {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "yes" => Ok(Self::YES),
            "no" => Ok(Self::NO),
            _ => Err("expected \"yes\" or \"no\""),
        }
    }
}

impl TryFrom<&str> for Bool {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for Bool {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.try_into()
    }
}

impl Display for Bool {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Serialize for Bool {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Bool {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(TryFromStringVisitor::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::{serde_test, value_from_str, value_to_string};
    use crate::Bool;

    #[test]
    fn consts() {
        assert_eq!(Bool::YES, Bool(true));
        assert_eq!(Bool::NO, Bool(false));

        assert_eq!(Bool::yes(), Bool::YES);
        assert_eq!(Bool::no(), Bool::NO);
    }

    serde_test! {
        serde(value_to_string, value_from_str): {
            "yes" => Bool::YES,
            "no" => Bool::NO,
        }
    }
}
