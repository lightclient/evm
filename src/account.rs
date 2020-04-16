use primitive_types::U256;
use std::collections::BTreeMap;

pub struct Account {
    balance: U256,
    code: Vec<u8>,
    nonce: u64,
    storage: BTreeMap<U256, U256>,
}
