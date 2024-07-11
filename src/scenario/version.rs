use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::util::TryFromStringVisitor;

#[derive(Clone, Debug, Default)]
pub struct Version {
    epoch: usize,
    version: Range<usize>,
    revision: Range<usize>,
    original: String,
}

impl Version {
    pub fn epoch(&self) -> usize {
        self.epoch
    }

    pub fn version(&self) -> &str {
        &self.original[self.version.clone()]
    }

    pub fn revision(&self) -> &str {
        &self.original[self.revision.clone()]
    }

    pub fn as_str(&self) -> &str {
        &self.original
    }
}

impl AsRef<str> for Version {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.original, f)
    }
}

impl Eq for Version {}

impl PartialEq<Self> for Version {
    fn eq(&self, other: &Self) -> bool {
        self.epoch == other.epoch
            && self.version() == other.version()
            && self.revision() == other.revision()
    }
}

impl std::hash::Hash for Version {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.epoch.hash(state);
        self.version().hash(state);
        self.revision().hash(state);
    }
}

fn cmp_non_digit(a: &mut &[u8], b: &mut &[u8]) -> Ordering {
    while !a.is_empty() || !b.is_empty() {
        match (
            a.first().filter(|c| !c.is_ascii_digit()),
            b.first().filter(|c| !c.is_ascii_digit()),
        ) {
            (None, None) => return Ordering::Equal,
            (Some(c_a), Some(c_b)) if c_a == c_b => {}
            (Some(b'~'), _) => return Ordering::Less,
            (_, Some(b'~')) => return Ordering::Greater,
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            (Some(c_a), Some(c_b)) => {
                if c_a != c_b {
                    return match (c_a.is_ascii_alphabetic(), c_b.is_ascii_alphabetic()) {
                        (true, true) | (false, false) => c_a.cmp(c_b),
                        (true, false) => Ordering::Less,
                        (false, true) => Ordering::Greater,
                    };
                }
            }
        }
        *a = &a[1..];
        *b = &b[1..];
    }

    Ordering::Equal
}

fn get_next_num(s: &mut &[u8]) -> u128 {
    std::iter::from_fn(|| match s.first() {
        Some(&c) if c.is_ascii_digit() => {
            *s = &s[1..];
            Some(c - b'0')
        }
        _ => None,
    })
    .fold(0, |num, digit| 10 * num + (digit as u128))
}

fn cmp_num(a: &mut &[u8], b: &mut &[u8]) -> Ordering {
    get_next_num(a).cmp(&get_next_num(b))
}

fn cmp_string(a: &str, b: &str) -> Ordering {
    let (mut a, mut b) = (a.as_bytes(), b.as_bytes());
    let mut compare_non_digit = true;

    while !a.is_empty() || !b.is_empty() {
        let res = if compare_non_digit {
            cmp_non_digit(&mut a, &mut b)
        } else {
            cmp_num(&mut a, &mut b)
        };

        if res != Ordering::Equal {
            return res;
        }
        compare_non_digit = !compare_non_digit;
    }

    Ordering::Equal
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.epoch > other.epoch {
            return Ordering::Greater;
        }

        if self.epoch < other.epoch {
            return Ordering::Less;
        }

        let version_cmp = cmp_string(self.version(), other.version());

        if version_cmp != Ordering::Equal {
            return version_cmp;
        }

        cmp_string(self.revision(), other.revision())
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl TryFrom<String> for Version {
    type Error = ParseIntError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let (epoch, epoch_len, remainder) = match value.split_once(':') {
            None => (0, 0, &*value),
            Some((epoch_str, remainder)) => (epoch_str.parse()?, epoch_str.len() + 1, remainder),
        };

        let (revision, remainder) = match remainder.rsplit_once('-') {
            None => (0..0, remainder),
            Some((remainder, revision_str)) => {
                ((value.len() - revision_str.len())..value.len(), remainder)
            }
        };

        Ok(Version {
            epoch,
            version: epoch_len..(epoch_len + remainder.len()),
            revision,
            original: value,
        })
    }
}

impl TryFrom<&str> for Version {
    type Error = ParseIntError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.to_string().try_into()
    }
}

impl Serialize for Version {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(TryFromStringVisitor::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod version {
        use std::cmp::Ordering::*;
        use std::num::IntErrorKind;

        use super::*;

        #[test]
        fn parse() {
            let all_components = Version::try_from("1:foo:bar-baz-qux").unwrap();
            assert_eq!(1, all_components.epoch());
            assert_eq!("foo:bar-baz", all_components.version());
            assert_eq!("qux", all_components.revision());

            let no_epoch = Version::try_from("foo.123+bar-baz-qux").unwrap();
            assert_eq!(0, no_epoch.epoch());
            assert_eq!("foo.123+bar-baz", no_epoch.version());
            assert_eq!("qux", no_epoch.revision());

            let no_revision = Version::try_from("90:foo.123+bar").unwrap();
            assert_eq!(90, no_revision.epoch());
            assert_eq!("foo.123+bar", no_revision.version());
            assert_eq!("", no_revision.revision());

            let no_epoch_and_revision = Version::try_from("foo.123+bar~baz").unwrap();
            assert_eq!(0, no_epoch_and_revision.epoch());
            assert_eq!("foo.123+bar~baz", no_epoch_and_revision.version());
            assert_eq!("", no_epoch_and_revision.revision());

            assert_eq!(
                &IntErrorKind::InvalidDigit,
                Version::try_from("foo:bar").unwrap_err().kind()
            )
        }

        #[test]
        fn cmp_string() {
            assert_eq!(
                Less,
                cmp_non_digit(&mut "~".as_bytes(), &mut "+".as_bytes())
            );
            assert_eq!(
                Greater,
                cmp_non_digit(&mut "~r".as_bytes(), &mut "~d".as_bytes())
            );
        }

        #[test]
        fn ord() {
            let source = vec![
                ("1.1.1", Less, "1.1.2"),
                ("1b", Greater, "1a"),
                ("1~~", Less, "1~~a"),
                ("1~~a", Less, "1~"),
                ("1", Less, "1.1"),
                ("1.0", Less, "1.1"),
                ("1.2", Less, "1.11"),
                ("1.0-1", Less, "1.1"),
                ("1.0-1", Less, "1.0-12"),
                // make them different for sorting
                ("1:1.0-0", Equal, "1:1.0"),
                ("1.0", Equal, "1.0"),
                ("1.0-1", Equal, "1.0-1"),
                ("1:1.0-1", Equal, "1:1.0-1"),
                ("1:1.0", Equal, "1:1.0"),
                ("1.0-1", Less, "1.0-2"),
                //("1.0final-5sarge1", Greater, "1.0final-5"),
                ("1.0final-5", Greater, "1.0a7-2"),
                ("0.9.2-5", Less, "0.9.2+cvs.1.0.dev.2004.07.28-1"),
                ("1:500", Less, "1:5000"),
                ("100:500", Greater, "11:5000"),
                ("1.0.4-2", Greater, "1.0pre7-2"),
                ("1.5~rc1", Less, "1.5"),
                ("1.5~rc1", Less, "1.5+1"),
                ("1.5~rc1", Less, "1.5~rc2"),
                ("1.5~rc1", Greater, "1.5~dev0"),
            ];

            for e in source {
                assert_eq!(
                    Version::try_from(e.0)
                        .unwrap()
                        .cmp(&Version::try_from(e.2).unwrap()),
                    e.1,
                    "{:#?} vs {:#?}",
                    Version::try_from(e.0).unwrap(),
                    Version::try_from(e.2).unwrap()
                );
            }
        }

        #[test]
        fn eq() {
            let source = vec![("1.1+git2021", "0:1.1+git2021")];
            for e in &source {
                assert_eq!(
                    Version::try_from(e.0).unwrap(),
                    Version::try_from(e.1).unwrap()
                );
            }
        }
    }
}
