use primitive_types::U256;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub struct Account {
    pub balance: U256,
    pub code: Vec<u8>,
    pub nonce: u64,
    pub storage: BTreeMap<U256, U256>,
}
