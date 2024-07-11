use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::util::TryFromStringVisitor;

/// A [`bool`] wrapper type that serializes `true` and `false` to `"yes"` and `"no"` respectively.
/// [`Bool`] is generic over its default value `D`.
///
/// See [`Bool::serialize`] and [`Bool::deserialize`] for how the default value `D` is used to
/// serialize and deserialize empty/`None` values.
///
/// # Examples
/// ```
/// # use apt_edsp::Bool;
/// assert_eq!("yes", Bool::<false>::YES.as_str());
/// assert_eq!(Bool::<false>::NO, "no".parse().unwrap());
/// assert_eq!(Bool(true), Bool::<true>::default());
/// ```
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Bool<const D: bool = false>(pub bool);

impl<const D: bool> Bool<D> {
    /// Equivalent to `true` or `"yes"`.
    pub const YES: Self = Bool(true);

    /// Equivalent to `false` or `"no`".
    pub const NO: Self = Bool(false);

    /// Returns the [`Bool`] corresponding to `"yes"`.
    pub const fn yes() -> Self {
        Self::YES
    }

    /// Returns the [`Bool`] corresponding to `"no"`.
    pub const fn no() -> Self {
        Self::NO
    }

    /// Returns the string literal representation (`"yes"` or `"no"`) for this boolean value.
    pub const fn as_str(&self) -> &'static str {
        if self.0 {
            "yes"
        } else {
            "no"
        }
    }
}

impl<const D: bool> Default for Bool<D> {
    /// Uses the const generic parameter `D` as the default value and returns `Bool(D)`.
    fn default() -> Self {
        Self(D)
    }
}

impl<const D: bool> From<Bool<D>> for &'static str {
    fn from(value: Bool<D>) -> Self {
        value.as_str()
    }
}

impl<const D: bool> FromStr for Bool<D> {
    type Err = &'static str;

    /// Returns `Bool(true)` if `"yes"`, `Bool(false)` if `"no"`, otherwise returns [`Err`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "yes" => Ok(Self::YES),
            "no" => Ok(Self::NO),
            _ => Err("expected \"yes\" or \"no\""),
        }
    }
}

impl<const D: bool> TryFrom<&str> for Bool<D> {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl<const D: bool> TryFrom<String> for Bool<D> {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl<const D: bool> Display for Bool<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<const D: bool> Serialize for Bool<D> {
    /// Serializes [`Bool::<D>::default()`] to `None`, otherwise serializes the
    /// [string representation](Self::as_str).
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.0 != D {
            serializer.collect_str(self)
        } else {
            serializer.serialize_none()
        }
    }
}

impl<'de, const DEFAULT: bool> Deserialize<'de> for Bool<DEFAULT> {
    /// Deserializes an empty value to [`Bool::<DEFAULT>::default()`], otherwise deserializes
    /// [from the string representation](Self::from_str).
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor<const DEFAULT: bool>;

        impl<'de, const DEFAULT: bool> serde::de::Visitor<'de> for Visitor<DEFAULT> {
            type Value = Bool<DEFAULT>;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("yes or no, or nothing")
            }

            #[inline]
            fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(Bool::default())
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_str(TryFromStringVisitor::new())
            }

            #[inline]
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(Bool::default())
            }
        }

        deserializer.deserialize_option(Visitor::<DEFAULT>)
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::test_util::serde_test;

    use super::*;

    type BoolDefaultFalse = Bool<false>;

    #[test]
    fn consts() {
        assert_eq!(BoolDefaultFalse::YES, Bool(true));
        assert_eq!(BoolDefaultFalse::NO, Bool(false));

        assert_eq!(BoolDefaultFalse::yes(), BoolDefaultFalse::YES);
        assert_eq!(BoolDefaultFalse::no(), BoolDefaultFalse::NO);
    }

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    struct Test {
        foo: Bool<false>,
        bar: Bool<true>,
    }

    serde_test! {
        serde: {
            indoc! {"
                foo: yes
                bar: no
            "} => Test {
                foo: Bool::YES,
                bar: Bool::NO,
            },
            indoc! {"
                foo: yes
            "} => Test {
                foo: Bool::YES,
                bar: Bool::YES,
            },
            indoc! {"
                bar: no
            "} => Test {
                foo: Bool::NO,
                bar: Bool::NO,
            },
            "" => Test {
                foo: Bool::NO,
                bar: Bool::YES,
            },
        }
    }
}
