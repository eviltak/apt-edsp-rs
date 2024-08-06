use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::de::{Error, Visitor};

pub struct TryFromStringVisitor<T>(std::marker::PhantomData<T>);

impl<T> TryFromStringVisitor<T> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<'de, T> Visitor<'de> for TryFromStringVisitor<T>
where
    T: TryFrom<String, Error: Display> + for<'a> TryFrom<&'a str, Error: Display>,
{
    type Value = T;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string value (borrowed or owned)")
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        v.try_into().map_err(Error::custom)
    }

    fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
        v.try_into().map_err(Error::custom)
    }
}

pub struct FromStrVisitor<T>(std::marker::PhantomData<T>);

impl<T> FromStrVisitor<T> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}
impl<'de, T: FromStr<Err: Display>> Visitor<'de> for FromStrVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string value (borrowed or owned)")
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        v.parse().map_err(Error::custom)
    }

    fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
        v.parse().map_err(Error::custom)
    }
}

pub mod serde_as_string {
    use std::fmt::Display;
    use std::str::FromStr;

    use super::FromStrVisitor;

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr<Err: Display>,
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(FromStrVisitor::new())
    }

    pub fn serialize<T: ToString, S: serde::Serializer>(
        value: &T,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }
}
