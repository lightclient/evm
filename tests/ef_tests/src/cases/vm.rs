use crate::cases::{Case, LoadCase};
use crate::decode::{from_hex_to_buffer, from_hex_to_u64, json_decode_file};
use crate::error::Error;
use crate::fake_host::FakeHost;

use fast_evm::{
    account::Account,
    env::Environment,
    execute::execute,
    host::Host,
    message::{Inner as MessageBody, Message},
};

use primitive_types::{H160, U256};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::path::Path;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Vm {
    #[serde(rename = "_info")]
    pub info: Info,
    pub callcreates: Option<Vec<::serde_json::Value>>,
    pub env: TestEnvironment,
    pub exec: ExecutionContext,
    pub gas: Option<U256>,
    #[serde(skip_deserializing)]
    pub logs: Option<Vec<u8>>,
    #[serde(skip_deserializing)]
    pub out: Vec<u8>,
    pub post: Option<BTreeMap<H160, TestAccount>>,
    pub pre: BTreeMap<H160, TestAccount>,
}

impl Vm {
    pub fn pre_state(&self) -> BTreeMap<H160, Account> {
        let mut state = BTreeMap::<H160, Account>::new();

        for (h, a) in self.pre.iter() {
            state.insert(h.clone(), Account::from(a));
        }

        state
    }
}

impl LoadCase for Vm {
    fn load_from_dir(path: &Path) -> Result<Self, Error> {
        json_decode_file(path)
    }
}

impl Case for Vm {
    fn description(&self) -> String {
        self.info.comment.clone()
    }

    fn result(&self, _case_index: usize) -> Result<(), Error> {
        let mut host = FakeHost::with_state(self.pre_state());
        let msg = MessageBody::from(&self.exec);
        let code = host.get_code(&msg.target);
        let mut env = Environment::from(&self.env);
        env.gas_price = self.exec.gas_price;

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

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub comment: String,
    pub filledwith: String,
    pub lllcversion: String,
    pub source: String,
    pub source_hash: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct TestAccount {
    balance: U256,
    #[serde(deserialize_with = "from_hex_to_buffer")]
    code: Vec<u8>,
    #[serde(deserialize_with = "from_hex_to_u64")]
    nonce: u64,
    storage: BTreeMap<U256, U256>,
}

impl From<&TestAccount> for Account {
    fn from(a: &TestAccount) -> Account {
        Account {
            balance: a.balance,
            code: a.code.clone(),
            nonce: a.nonce,
            storage: a.storage.clone(),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TestEnvironment {
    pub current_coinbase: H160,
    pub current_difficulty: U256,
    pub current_gas_limit: U256,
    #[serde(deserialize_with = "from_hex_to_u64")]
    pub current_number: u64,
    #[serde(deserialize_with = "from_hex_to_u64")]
    pub current_timestamp: u64,
}

impl From<&TestEnvironment> for Environment {
    fn from(env: &TestEnvironment) -> Environment {
        Environment {
            coinbase: env.current_coinbase,
            difficulty: env.current_difficulty,
            gas_price: 0.into(),
            gas_limit: env.current_gas_limit,
            block_number: env.current_number.into(),
            timestamp: env.current_timestamp.into(),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionContext {
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

impl From<&ExecutionContext> for MessageBody {
    fn from(e: &ExecutionContext) -> MessageBody {
        MessageBody {
            target: e.address,
            caller: e.caller,
            origin: e.origin,
            value: e.value,
            data: e.data.clone(),
            gas: e.gas.as_u64(),
            depth: 0,
        }
    }
}
