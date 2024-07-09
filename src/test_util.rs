use std::fmt::Debug;

use serde::{Deserialize, Serialize};

pub struct TestCase<T> {
    repr: &'static str,
    val: T,
}

impl<T> TestCase<T> {
    pub fn new(repr: &'static str, val: T) -> Self {
        Self { repr, val }
    }
}

impl<T: Eq + Serialize + Deserialize<'static> + Debug> TestCase<T> {
    pub fn check<FS, FD>(&self, serialize_fn: FS, deserialize_fn: FD)
    where
        FS: Fn(&T) -> String,
        FD: Fn(&'static str) -> T,
    {
        assert_eq!(
            self.val,
            (deserialize_fn)(self.repr),
            "Incorrect deserialized value from '{}' (left: expected, right: actual)",
            self.repr
        );
        assert_eq!(
            self.repr,
            (serialize_fn)(&self.val),
            "Incorrect serialized value from '{:?}' (left: expected, right: actual)",
            self.val
        );
    }
}

pub fn struct_to_string<T: Serialize + Debug>(val: &T) -> String {
    match rfc822_like::to_string(val) {
        Ok(t) => t,
        Err(e) => panic!("Error when serializing {val:?}: {e}"),
    }
}

pub fn struct_from_str<'de, T: Deserialize<'de>>(s: &'de str) -> T {
    match rfc822_like::from_str(s) {
        Ok(t) => t,
        Err(e) => panic!("Error when deserializing \"{s}\": {e}"),
    }
}

pub fn value_to_string<T: Serialize + Debug>(val: &T) -> String {
    #[derive(Serialize, Debug)]
    struct Record<'a, V> {
        xxx: &'a V,
    }

    struct_to_string(&Record { xxx: val }).trim()["xxx: ".len()..].to_string()
}

pub fn value_from_str<T: for<'de> Deserialize<'de>>(s: &str) -> T {
    #[derive(Deserialize)]
    struct Record<V> {
        xxx: V,
    }

    struct_from_str::<Record<T>>(&format!("xxx: {s}")).xxx
}

#[macro_export]
macro_rules! serde_test {
    ($name:ident: {$($repr:expr => $val:expr),+ $(,)?}) => {
        serde_test!(
            $name(
            crate::test_util::struct_to_string,
            crate::test_util::struct_from_str
            ): {$($repr => $val),+}
        );
    };

    ($name:ident($serialize_fn:expr, $deserialize_fn:expr): {$($repr:expr => $val:expr),+ $(,)?}) => {
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
                crate::test_util::TestCase::new(repr, val).check($serialize_fn, $deserialize_fn);
            }
            )+
        }
    };
}

pub use serde_test;