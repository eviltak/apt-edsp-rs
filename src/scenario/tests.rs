use indoc::indoc;

use crate::test_util::{serde_test, value_from_str, value_to_string};

use super::*;

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
                upgrade_all: Bool::YES,
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
        "} =>
        vec![
            Request {
                request: "EDSP 0.5".into(),
                architecture: "amd64".into(),
                actions: Actions {
                    upgrade_all: Bool::YES,
                    ..Default::default()
                },
                ..Default::default()
            },
            Request {
                request: "EDSP 0.5".into(),
                architecture: "amd64".into(),
                actions: Actions {
                    upgrade_all: Bool::NO,
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
