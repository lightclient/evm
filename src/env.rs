use primitive_types::{H160, U256};

pub struct Environment {
    // chain info
    pub chain_id: U256,
    pub coinbase: H160,
    pub difficulty: U256,
    pub block_number: U256,
    pub timestamp: U256,
    pub gas_limit: U256,

    // tx context
    pub gas_price: U256,
    pub origin: H160,
}

impl From<evmc_vm::ffi::evmc_tx_context> for Environment {
    fn from(ctx: evmc_vm::ffi::evmc_tx_context) -> Environment {
        Environment {
            chain_id: U256::from_big_endian(&ctx.chain_id.bytes),
            coinbase: H160::from_slice(&ctx.block_coinbase.bytes),
            difficulty: U256::from_big_endian(&ctx.block_difficulty.bytes),
            block_number: U256::from(ctx.block_number),
            timestamp: U256::from(ctx.block_timestamp),
            gas_limit: U256::from(ctx.block_gas_limit),
            gas_price: U256::from_big_endian(&ctx.tx_gas_price.bytes),
            origin: H160::from_slice(&ctx.tx_origin.bytes),
        }
    }
}
