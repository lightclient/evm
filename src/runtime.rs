use crate::account::Account;
use crate::env::Environment;

use primitive_types::H160;
use std::collections::BTreeMap;

pub struct Runtime {
    state: BTreeMap<H160, Account>,
    env: Environment,
}
