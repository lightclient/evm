use crate::cases::{Case, LoadCase};
use crate::decode::{from_hex_to_buffer, from_hex_to_u64, json_decode_file};
use crate::error::Error;

use fast_evm::{
    env::Environment,
    execute::execute,
    host::Host,
    message::{Inner as MessageBody, Message},
};

use log::error;
use primitive_types::{H160, U256};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::path::Path;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Account {
    balance: U256,
    #[serde(deserialize_with = "from_hex_to_buffer")]
    code: Vec<u8>,
    #[serde(deserialize_with = "from_hex_to_u64")]
    nonce: u64,
    storage: BTreeMap<U256, U256>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Vm {
    #[serde(rename = "_info")]
    pub info: Info,
    pub callcreates: Option<Vec<::serde_json::Value>>,
    pub env: Env,
    pub exec: Exec,
    pub gas: Option<U256>,
    #[serde(skip_deserializing)]
    pub logs: Option<Vec<u8>>,
    #[serde(skip_deserializing)]
    pub out: Vec<u8>,
    pub post: Option<BTreeMap<H160, Account>>,
    pub pre: BTreeMap<H160, Account>,
}

impl LoadCase for Vm {
    fn load_from_dir(path: &Path) -> Result<Self, Error> {
        json_decode_file(path)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub comment: String,
    pub filledwith: String,
    pub lllcversion: String,
    pub source: String,
    pub source_hash: String,
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Env {
    pub current_coinbase: H160,
    pub current_difficulty: U256,
    pub current_gas_limit: U256,
    #[serde(deserialize_with = "from_hex_to_u64")]
    pub current_number: u64,
    #[serde(deserialize_with = "from_hex_to_u64")]
    pub current_timestamp: u64,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Exec {
    pub address: H160,
    pub caller: H160,
    #[serde(deserialize_with = "from_hex_to_buffer")]
    pub code: Vec<u8>,
    #[serde(deserialize_with = "from_hex_to_buffer")]
    pub data: Vec<u8>,
    pub gas: U256,
    pub gas_price: U256,
    pub origin: H160,
    pub value: U256,
}

impl Case for Vm {
    fn description(&self) -> String {
        self.info.comment.clone()
    }

    fn result(&self, _case_index: usize) -> Result<(), Error> {
        // error!("{:?}\n", self.info);

        let env = Environment {
            coinbase: self.env.current_coinbase,
            difficulty: self.env.current_difficulty,
            gas_price: self.exec.gas_price,
            gas_limit: self.env.current_gas_limit,
            block_number: self.env.current_number.into(),
            timestamp: self.env.current_timestamp.into(),
        };

        let mut state = BTreeMap::<H160, fast_evm::account::Account>::new();
        for (h, a) in self.pre.iter() {
            let a = fast_evm::account::Account {
                balance: a.balance,
                code: a.code.clone(),
                nonce: a.nonce,
                storage: a.storage.clone(),
            };

            state.insert(h.clone(), a);
        }

        let msg = MessageBody {
            target: self.exec.address,
            caller: self.exec.caller,
            origin: self.exec.origin,
            value: self.exec.value,
            data: self.exec.data.clone(),
            gas: self.exec.gas.as_u64(),
            depth: 0,
        };

        let mut host = FakeHost::with_state(state);

        let code = match host.state.get(&msg.target) {
            Some(a) => a.code.clone(),
            None => return Err(Error::NotEqual("target missing from state".into())),
        };

        let _ = execute(&mut host, &env, Message::Call(msg), &code);

        if self.post.is_none() {
            return Ok(());
        }

        let post = self.post.clone().unwrap();

        for (address, expected) in post {
            let actual = host.state.get(&address);

            if actual.is_none() {
                return Err(Error::NotEqual(format!(
                    "Couldn't load account from json: {:?}",
                    address
                )));
            }

            let actual = actual.unwrap();

            if actual.code != expected.code {
                return Err(Error::NotEqual(format!(
                    "Got code: {:?}, expected: {:?}",
                    actual.code, expected.code
                )));
            }

            if actual.nonce != expected.nonce {
                return Err(Error::NotEqual(format!(
                    "Got nonce: {:?}, expected: {:?}",
                    actual.nonce, expected.nonce
                )));
            }

            for (k, ev) in expected.storage.clone() {
                match actual.storage.get(&k) {
                    Some(av) => {
                        if ev != *av {
                            return Err(Error::NotEqual(format!(
                                "Storage mismatch at {:?}. Got: {:?}, expected: {:?}",
                                k, av, ev
                            )));
                        }
                    }
                    None => {
                        return Err(Error::NotEqual(format!(
                            "Storage missing at {:?}. Expected: {:?}",
                            k, ev
                        )))
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct FakeHost {
    state: BTreeMap<H160, fast_evm::account::Account>,
}

impl FakeHost {
    pub fn with_state(state: BTreeMap<H160, fast_evm::account::Account>) -> Self {
        Self { state }
    }
}

impl Host for FakeHost {
    fn get_storage(&self, address: &H160, key: &U256) -> U256 {
        let account = self.state.get(address).unwrap();
        match account.storage.get(key) {
            Some(v) => *v,
            None => 0.into(),
        }
    }

    fn set_storage(&mut self, address: &H160, key: U256, value: U256) {
        let account = self.state.get_mut(address).unwrap();
        account.storage.insert(key, value);
    }

    fn self_destruct(&mut self, address: &H160, beneficiary: H160) {
        let account = self.state.remove(address).unwrap();
        let target = self.state.entry(beneficiary).or_default();
        target.balance += account.balance;
    }
}
