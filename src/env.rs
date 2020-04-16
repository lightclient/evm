use primitive_types::H160;

pub struct Environment {
    coinbase: H160,
    difficulty: u64,
    gas_limit: u64,
    block_number: u64,
    timestamp: u64,
}
