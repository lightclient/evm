use crate::cases::Case;
use crate::error::Error;

use std::fmt::Debug;
use std::path::{Path, PathBuf};

pub const MAX_VALUE_STRING_LEN: usize = 500;

#[derive(Debug, PartialEq, Clone)]
pub struct CaseResult {
    pub case_index: usize,
    pub desc: String,
    pub path: PathBuf,
    pub result: Result<(), Error>,
}

impl CaseResult {
    pub fn new(
        case_index: usize,
        path: &Path,
        case: &impl Case,
        result: Result<(), Error>,
    ) -> Self {
        CaseResult {
            case_index,
            desc: case.description(),
            path: path.into(),
            result,
        }
    }
}

/// Compares `result` with `expected`.
///
/// If `expected.is_none()` then `result` is expected to be `Err`. Otherwise, `T` in `result` and
/// `expected` must be equal.
pub fn compare_result<T, E>(result: &Result<T, E>, expected: &Option<T>) -> Result<(), Error>
where
    T: PartialEq<T> + Debug,
    E: Debug,
{
    match (result, expected) {
        // Pass: The should have failed and did fail.
        (Err(_), None) => Ok(()),
        // Fail: The test failed when it should have produced a result (fail).
        (Err(e), Some(expected)) => Err(Error::NotEqual(format!(
            "Got {:?} | Expected {:?}",
            e,
            fmt_val(expected)
        ))),
        // Fail: The test produced a result when it should have failed (fail).
        (Ok(result), None) => Err(Error::DidntFail(format!("Got {:?}", fmt_val(result)))),
        // Potential Pass: The test should have produced a result, and it did.
        (Ok(result), Some(expected)) => {
            if result == expected {
                Ok(())
            } else {
                Err(Error::NotEqual(format!(
                    "Got {:?} | Expected {:?}",
                    fmt_val(result),
                    fmt_val(expected)
                )))
            }
        }
    }
}

fn fmt_val<T: Debug>(val: T) -> String {
    let mut string = format!("{:?}", val);
    string.truncate(MAX_VALUE_STRING_LEN);
    string
}
