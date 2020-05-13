use primitive_types::{H160, U256};

pub trait Host {
    // fn call(msg: Message) ->
    fn get_storage(&self, address: &H160, key: &U256) -> U256;
    fn set_storage(&mut self, address: &H160, key: U256, value: U256);
    fn get_code(&self, address: &H160) -> Vec<u8>;
    fn self_destruct(&mut self, address: &H160, beneficiary: H160);
}
