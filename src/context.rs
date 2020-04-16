use primitive_types::{H160, U256};

pub struct Context {
    target: H160,
    caller: H160,
    origin: H160,
    value: U256,
}
