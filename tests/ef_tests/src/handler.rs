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

    fn run() {
        Self::run_specified_tests(None);
    }

    fn run_specified_tests<T: Into<Option<Vec<&'static str>>>>(tests: T) {
        let tests: Option<Vec<&str>> = tests.into();

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
                        if let Some(tests) = tests.clone() {
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

macro_rules! make_handler {
    ($i: ident, $n: expr) => {
        pub struct $i;

        impl Handler for $i {
            type Case = cases::Vm;

            fn runner_name() -> &'static str {
                "VM"
            }

            fn handler_name() -> String {
                $n.into()
            }
        }
    };
}

make_handler!(VmArithmeticHandler, "ArithmeticTest");
make_handler!(VmBitwiseOperationsHandler, "BitwiseLogicOperation");
make_handler!(VmBlockInfoHandler, "BlockInfoTest");
make_handler!(VmEnvironmentalInfoHandler, "EnvironmentalInfo");
make_handler!(VmIoAndFlowHandler, "IOandFlowOperations");
make_handler!(VmLogHandler, "LogTest");
make_handler!(VmPerformanceHandler, "Performance");
make_handler!(VmPushDupSwapHandler, "PushDupSwapTest");
make_handler!(VmRandomHandler, "RandomTest");
make_handler!(VmSha3Handler, "Sha3Test");
make_handler!(VmSystemOperationsHandler, "SystemOperations");
make_handler!(VmTestsHandler, "Tests");
