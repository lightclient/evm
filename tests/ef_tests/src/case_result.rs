use crate::cases::Case;
use crate::error::Error;

use std::fmt::Debug;
use std::path::{Path, PathBuf};

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
