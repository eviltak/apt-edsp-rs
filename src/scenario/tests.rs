use std::fmt::Debug;

use indoc::indoc;

use super::*;

struct TestCase<T> {
    repr: &'static str,
    val: T,
}

impl<T: Eq + Serialize + Deserialize<'static> + std::fmt::Debug> TestCase<T> {
    fn check<FS, FD>(&self, serialize_fn: FS, deserialize_fn: FD)
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

fn struct_to_string<T: Serialize + Debug>(val: &T) -> String {
    match rfc822_like::to_string(val) {
        Ok(t) => t,
        Err(e) => panic!("Error when serializing {val:?}: {e}"),
    }
}

fn struct_from_str<'de, T: Deserialize<'de>>(s: &'de str) -> T {
    match rfc822_like::from_str(s) {
        Ok(t) => t,
        Err(e) => panic!("Error when deserializing \"{s}\": {e}"),
    }
}

fn value_to_string<T: Serialize + Debug>(val: &T) -> String {
    #[derive(Serialize, Debug)]
    struct Record<'a, V> {
        xxx: &'a V,
    }

    struct_to_string(&Record { xxx: val }).trim()["xxx: ".len()..].to_string()
}

fn value_from_str<T: for<'de> Deserialize<'de>>(s: &str) -> T {
    #[derive(Deserialize)]
    struct Record<V> {
        xxx: V,
    }

    struct_from_str::<Record<T>>(&format!("xxx: {s}")).xxx
}

macro_rules! serde_test {
    ($name:ident: {$($repr:expr => $val:expr),+}) => {
        serde_test!(
            $name(
            struct_to_string,
            struct_from_str
            ): {$($repr => $val),+}
        );
    };

    ($name:ident($serialize_fn:expr, $deserialize_fn:expr): {$($repr:expr => $val:expr),+}) => {
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
                TestCase {
                    repr,
                    val,
                }.check($serialize_fn, $deserialize_fn);
            }
            )+
        }
    };
}

serde_test! {
    request: {
        indoc! {"
            Request: EDSP 0.5
            Architecture: amd64
            Upgrade-All: yes
        "} =>
        Request {
            request: "EDSP 0.5".into(),
            architecture: "amd64".into(),
            actions: Actions {
                upgrade_all: Some("yes".into()),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

serde_test! {
    vec_request: {
        indoc! {"
            Request: EDSP 0.5
            Architecture: amd64
            Upgrade-All: yes

            Request: EDSP 0.5
            Architecture: amd64
            Upgrade-All: no
        "} =>
        vec![
            Request {
                request: "EDSP 0.5".into(),
                architecture: "amd64".into(),
                actions: Actions {
                    upgrade_all: Some("yes".into()),
                    ..Default::default()
                },
                ..Default::default()
            },
            Request {
                request: "EDSP 0.5".into(),
                architecture: "amd64".into(),
                actions: Actions {
                    upgrade_all: Some("no".into()),
                    ..Default::default()
                },
                ..Default::default()
            }
        ]
    }
}

serde_test! {
    relationship(value_to_string, value_from_str): {
        "foo" =>
        Relationship {
            package: "foo".into(),
            constraint: None,
        },
        "foo (<< 2.2.1)" =>
        Relationship {
            package: "foo".into(),
            constraint: Some((Relation::Earlier, Version::try_from("2.2.1").unwrap())),
        },
        "foo (<= 2.2.1)" =>
        Relationship {
            package: "foo".into(),
            constraint: Some((Relation::EarlierEqual, Version::try_from("2.2.1").unwrap())),
        },
        "foo (= 2.2.1)" =>
        Relationship {
            package: "foo".into(),
            constraint: Some((Relation::Equal, Version::try_from("2.2.1").unwrap())),
        },
        "foo (>= 2.2.1)" =>
        Relationship {
            package: "foo".into(),
            constraint: Some((Relation::LaterEqual, Version::try_from("2.2.1").unwrap())),
        },
        "foo (>> 2.2.1)" =>
        Relationship {
            package: "foo".into(),
            constraint: Some((Relation::Later, Version::try_from("2.2.1").unwrap())),
        }
    }
}

serde_test! {
    vec_relationship(value_to_string, value_from_str): {
        indoc! {"
            foo,
                 bar,
                 baz
        "}.trim() =>
        vec![
            Relationship {
                package: "foo".into(),
                constraint: None,
            },
            Relationship {
                package: "bar".into(),
                constraint: None,
            },
            Relationship {
                package: "baz".into(),
                constraint: None,
            }
        ]
    }
}

serde_test! {
    dependency(value_to_string, value_from_str): {
        "foo" =>
        Dependency {
            first: Relationship {
                package: "foo".into(),
                constraint: None,
            },
            alternates: vec![],
        },
        "foo (= v1.0.0) | bar | baz (>> 0.1~1)" =>
        Dependency {
            first: Relationship {
                package: "foo".into(),
                constraint: Some((Relation::Equal, Version::try_from("v1.0.0").unwrap())),
            },
            alternates: vec![
                Relationship {
                    package: "bar".into(),
                    constraint: None,
                },
                Relationship {
                    package: "baz".into(),
                    constraint: Some((Relation::Later, Version::try_from("0.1~1").unwrap())),
                },
            ],
        }
    }
}

serde_test! {
    vec_dependencies(value_to_string, value_from_str): {
        indoc! {"
            foo (= v1.0.0) | bar,
                 baz,
                 qux | quux (>> 0.1~1)
        "}.trim() =>
        vec![
            Dependency {
                first: Relationship {
                    package: "foo".into(),
                    constraint: Some((Relation::Equal, Version::try_from("v1.0.0").unwrap())),
                },
                alternates: vec![
                    Relationship {
                        package: "bar".into(),
                        constraint: None,
                    },
                ],
            },
            Dependency {
                first: Relationship {
                    package: "baz".into(),
                    constraint: None,
                },
                alternates: vec![],
            },
            Dependency {
                first: Relationship {
                    package: "qux".into(),
                    constraint: None,
                },
                alternates: vec![
                    Relationship {
                        package: "quux".into(),
                        constraint: Some((Relation::Later, Version::try_from("0.1~1").unwrap())),
                    },
                ],
            },
        ]
    }
}
