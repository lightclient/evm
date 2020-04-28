use primitive_types::{H160, U256};

pub struct Environment {
    pub coinbase: H160,
    pub difficulty: U256,
    pub gas_limit: U256,
    pub gas_price: U256,
    pub block_number: U256,
    pub timestamp: U256,
}
