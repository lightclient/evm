use primitive_types::{H160, U256};

pub struct Context {
    pub target: H160,
    pub caller: H160,
    pub origin: H160,
    pub value: U256,
    pub data: Vec<u8>,
}
