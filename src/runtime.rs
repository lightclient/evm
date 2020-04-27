use crate::account::Account;
use crate::ctx::Context;
use crate::env::Environment;
use crate::interupt::{Interupt, Yield};
use crate::machine::Machine;

use primitive_types::H160;
use std::collections::BTreeMap;

pub struct Runtime {
    pub state: BTreeMap<H160, Account>,
    pub env: Environment,
}

impl Runtime {
    pub fn execute(&mut self, ctx: Context) {
        let mut m = Machine::new(
            self.state.get(&ctx.target).unwrap().code.clone(),
            ctx.clone(),
            &self.env,
        );

        loop {
            match m.run() {
                Interupt::Yield(y) => match y {
                    Yield::Store(k, v) => {
                        let account = self.state.get_mut(&ctx.target).unwrap();
                        account.storage.insert(k, v);
                    }
                    _ => unimplemented!(),
                },
                _ => break,
            }
        }
    }
}
