use crate::cases::{self, Case, Cases, LoadCase};
use crate::results::assert_tests_pass;

use std::fs;
use std::path::PathBuf;

pub trait Handler {
    type Case: Case + LoadCase;

    fn config_name() -> &'static str {
        "ef-tests"
    }

    fn runner_name() -> &'static str;

    fn handler_name() -> String;

    fn run(whitelist: Option<Vec<&str>>) {
        let handler_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(format!("{}{}", Self::runner_name(), "Tests"))
            .join(format!("{}{}", "vm", Self::handler_name()));

        let test_cases = fs::read_dir(&handler_path)
            .expect("handler dir exists")
            .flat_map(|entry| {
                entry
                    .ok()
                    .filter(|e| e.file_type().map(|ty| !ty.is_dir()).unwrap_or(false))
                    .filter(|e| {
                        if let Some(tests) = whitelist.clone() {
                            tests.contains(&e.file_name().to_str().unwrap())
                        } else {
                            true
                        }
                    })
            })
            .map(|test_case| {
                let path = test_case.path();
                let case = Self::Case::load_from_dir(&path).expect("test should load");
                (path, case)
            })
            .collect();

        let results = Cases { test_cases }.test_results();

        let name = format!("{}/{}", Self::runner_name(), Self::handler_name());
        assert_tests_pass(&name, &handler_path, &results);
    }
}

pub struct VmHandler;

impl Handler for VmHandler {
    type Case = cases::Vm;

    fn runner_name() -> &'static str {
        "VM"
    }

    fn handler_name() -> String {
        "ArithmeticTest".into()
    }
}
