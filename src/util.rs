use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::de::{Error, Visitor};
use serde::Deserializer;

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

pub struct SpaceSeparatedFromStrVisitor<C, T>(
    std::marker::PhantomData<C>,
    std::marker::PhantomData<T>,
);

impl<C, T> SpaceSeparatedFromStrVisitor<C, T>
where
    C: FromIterator<T>,
    T: FromStr<Err: Display>,
{
    pub fn new() -> Self {
        Self(std::marker::PhantomData, std::marker::PhantomData)
    }

    fn visit(self, s: &str) -> Result<C, T::Err> {
        s.split_ascii_whitespace().map(str::parse).collect()
    }
}

impl<'de, C, T> Visitor<'de> for SpaceSeparatedFromStrVisitor<C, T>
where
    C: FromIterator<T>,
    T: FromStr<Err: Display>,
{
    type Value = C;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string value (borrowed or owned)")
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        self.visit(v).map_err(Error::custom)
    }

    fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
        self.visit(&v).map_err(Error::custom)
    }

    #[inline]
    fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
        Ok(std::iter::empty().collect())
    }

    #[inline]
    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_str(self)
    }

    #[inline]
    fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
        self.visit_none()
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

    pub fn serialize<T: Display, S: serde::Serializer>(
        value: &T,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_str(value)
    }
}

pub mod serde_space_separated_as_string {
    use itertools::Itertools;
    use std::fmt::Display;
    use std::str::FromStr;

    use super::SpaceSeparatedFromStrVisitor;

    pub fn serialize<'a, T: Display + 'a, S: serde::Serializer>(
        value: impl IntoIterator<Item = &'a T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut iter = value.into_iter().peekable();
        if iter.peek().is_some() {
            serializer.collect_str(&iter.format(" "))
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, C, T, D>(deserializer: D) -> Result<C, D::Error>
    where
        C: FromIterator<T>,
        T: FromStr<Err: Display>,
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_option(SpaceSeparatedFromStrVisitor::new())
    }
}
