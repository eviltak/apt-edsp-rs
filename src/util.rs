use std::fmt::{Display, Formatter};
use serde::de::{Error, Visitor};

pub struct TryFromStringVisitor<T>(std::marker::PhantomData<T>);

impl<T> TryFromStringVisitor<T> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<'de, T> Visitor<'de> for TryFromStringVisitor<T>
where
    T: TryFrom<String, Error: Display> + for<'a> TryFrom<&'a str, Error: Display>
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
