mod vm;
pub use vm::*;

use crate::case_result::CaseResult;
use crate::error::Error;

use std::fmt::Debug;
use std::path::{Path, PathBuf};

pub trait Case: Debug + Sync {
    fn description(&self) -> String {
        "no description".to_string()
    }

    fn result(&self, case_index: usize) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct Cases<T> {
    pub test_cases: Vec<(PathBuf, T)>,
}

impl<T: Case> Cases<T> {
    pub fn test_results(&self) -> Vec<CaseResult> {
        self.test_cases
            .iter()
            .enumerate()
            .map(|(i, (ref path, ref tc))| CaseResult::new(i, path, tc, tc.result(i)))
            .collect()
    }
}

pub trait LoadCase: Sized {
    fn load_from_dir(_path: &Path) -> Result<Self, Error>;
}
