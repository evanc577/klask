use super::AppState;
use crate::arg_state::{ArgKind, ArgState};
use clap::{Clap, FromArgMatches, IntoApp, ValueHint};
use std::{fmt::Debug, path::PathBuf};
use uuid::Uuid;

#[derive(Debug, Clap, PartialEq, Eq)]
struct ForbidEmpty {
    #[clap(long, forbid_empty_values = true)]
    optional_no_empty1: Option<String>,
    #[clap(long, forbid_empty_values = true)]
    optional_no_empty2: Option<String>,
    #[clap(long, forbid_empty_values = true)]
    optional_no_empty3: Option<String>,
}

#[test]
fn forbid_empty() {
    test_app(
        |args| {
            args[0].enter("a");
            args[2].enter("");
        },
        ForbidEmpty {
            optional_no_empty1: Some("a".into()),
            optional_no_empty2: None,
            optional_no_empty3: None,
        },
    );
}

#[derive(Debug, Clap, PartialEq, Eq)]
struct OptionalAndDefault {
    required: String,
    optional: Option<String>,
    #[clap(default_value = "d")]
    default: String,
}

#[test]
fn optional_and_default() {
    test_app(
        |args| args[0].enter("a"),
        OptionalAndDefault {
            required: "a".into(),
            optional: None,
            default: "d".into(),
        },
    );
}

#[derive(Debug, Clap, PartialEq, Eq)]
struct UseEquals {
    #[clap(long, require_equals = true)]
    long: String,
    #[clap(short, require_equals = true)]
    short: String,
    #[clap(long, require_equals = true, value_hint = ValueHint::AnyPath)]
    path: PathBuf,
    #[clap(long, require_equals = true, possible_values = &["P", "O"])]
    choose: String,
    #[clap(long, require_equals = true, multiple_occurrences = true)]
    multiple: Vec<String>,
    #[clap(long, parse(from_occurrences))]
    occurrences: i32,
    #[clap(long)]
    flag: bool,
}

#[test]
fn use_equals() {
    test_app(
        |args| {
            enter_consecutive(args, ["a", "b", "c", "P"]);
            args[4].enter_multiple(["d", "e"]);
            args[5].occurrences(3);
            args[6].set();
        },
        UseEquals {
            long: "a".into(),
            short: "b".into(),
            path: "c".into(),
            choose: "P".into(),
            multiple: vec!["d".into(), "e".into()],
            occurrences: 3,
            flag: true,
        },
    );
}

fn test_app<C, F>(setup: F, expected: C)
where
    C: IntoApp + FromArgMatches + Debug + Eq,
    F: FnOnce(&mut Vec<ArgState>),
{
    let app = C::into_app();
    let mut app_state = AppState::new(&app);
    setup(&mut app_state.args);
    let args = app_state.get_cmd_args(vec!["_name".into()]).unwrap();
    eprintln!("Args: {:?}", &args[1..]);
    let matches = app.try_get_matches_from(args.iter()).unwrap();
    let c = C::from_arg_matches(&matches).unwrap();
    assert_eq!(c, expected);
}

fn enter_consecutive<const N: usize>(args: &mut Vec<ArgState>, vals: [&str; N]) {
    for i in 0..N {
        args[i].enter(vals[i]);
    }
}

impl crate::arg_state::ArgState {
    fn enter(&mut self, val: &str) {
        if let ArgKind::String { value, .. } = &mut self.kind {
            value.0 = val.to_string();
        } else {
            panic!("Called enter on {:?}", self)
        }
    }

    fn enter_multiple<const N: usize>(&mut self, vals: [&str; N]) {
        if let ArgKind::MultipleStrings { values, .. } = &mut self.kind {
            *values = vals
                .iter()
                .map(|s| (s.to_string(), Uuid::new_v4()))
                .collect()
        } else {
            panic!("Called enter_multiple on {:?}", self)
        }
    }

    fn occurrences(&mut self, val: i32) {
        if let ArgKind::Occurences(i) = &mut self.kind {
            *i = val;
        } else {
            panic!("Called occurrences on {:?}", self)
        }
    }

    fn set(&mut self) {
        if let ArgKind::Bool(b) = &mut self.kind {
            *b = true;
        } else {
            panic!("Called set on {:?}", self)
        }
    }
}
