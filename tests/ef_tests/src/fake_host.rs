use fast_evm::{account::Account, host::Host};
use primitive_types::{H160, U256};
use std::collections::BTreeMap;

pub struct FakeHost {
    pub state: BTreeMap<H160, fast_evm::account::Account>,
}

impl FakeHost {
    pub fn with_state(state: BTreeMap<H160, Account>) -> Self {
        Self { state }
    }
}

impl Host for FakeHost {
    fn get_storage(&self, address: &H160, key: &U256) -> U256 {
        let account = self
            .state
            .get(address)
            .expect("getting storage from uninitialized account");

        match account.storage.get(key) {
            Some(v) => *v,
            None => 0.into(),
        }
    }

    fn set_storage(&mut self, address: &H160, key: U256, value: U256) {
        let account = self
            .state
            .get_mut(address)
            .expect("setting storage on uninitialized account");

        account.storage.insert(key, value);
    }

    fn get_code(&self, address: &H160) -> Vec<u8> {
        let account = self.state.get(address).unwrap();
        account.code.clone()
    }

    fn self_destruct(&mut self, address: &H160, beneficiary: H160) {
        let account = self.state.remove(address).unwrap();
        let target = self.state.entry(beneficiary).or_default();
        target.balance += account.balance;
    }
}
