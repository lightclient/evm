use crate::cases::{Case, LoadCase};
use crate::decode::{from_hex_to_buffer, from_hex_to_u64, json_decode_file};
use crate::error::Error;

use fast_evm::{ctx::Context, env::Environment, runtime::Runtime};
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
        let env = Environment {
            coinbase: self.env.current_coinbase,
            difficulty: self.env.current_difficulty,
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

        let mut rt = Runtime { env, state };

        let ctx = Context {
            target: self.exec.address,
            caller: self.exec.caller,
            origin: self.exec.origin,
            value: self.exec.value,
            data: self.exec.data.clone(),
        };

        rt.execute(ctx);

        if self.post.is_none() {
            return Ok(());
        }

        let post = self.post.clone().unwrap();

        for (h, a) in rt.state {
            let b = post.get(&h);

            if b.is_none() {
                return Err(Error::NotEqual(format!(
                    "Couldn't load account from json: {}",
                    h
                )));
            }

            let b = b.unwrap();

            if a.balance != b.balance {
                return Err(Error::NotEqual(format!(
                    "Got balance: {:?}, expected: {:?}",
                    a.balance, b.balance
                )));
            }

            if a.code != b.code {
                return Err(Error::NotEqual(format!(
                    "Got code: {:?}, expected: {:?}",
                    a.code, b.code
                )));
            }

            if a.nonce != b.nonce {
                return Err(Error::NotEqual(format!(
                    "Got nonce: {:?}, expected: {:?}",
                    a.nonce, b.nonce
                )));
            }

            for (k, v) in b.storage.clone() {
                match a.storage.get(&k) {
                    Some(av) => {
                        if v != *av {
                            return Err(Error::NotEqual(format!(
                                "Storage mismatch at {:?}. Got: {:?}, expected: {:?}",
                                k, av, v
                            )));
                        }
                    }
                    None => {
                        return Err(Error::NotEqual(format!(
                            "Storage missing at {:?}. Expected: {:?}",
                            k, v
                        )))
                    }
                }
            }
        }

        Ok(())
    }
}
