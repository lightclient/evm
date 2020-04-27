use crate::account::Account;
use crate::ctx::Context;
use crate::env::Environment;
use crate::interupt::{Interupt, Yield};
use crate::machine::Machine;

use log::info;
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
            let i = m.run();
            info!("interupt reason: {:?}", i);

            match i {
                Interupt::Yield(y) => match y {
                    Yield::Load(k) => {
                        let account = self.state.get_mut(&ctx.target).unwrap();
                        match account.storage.get(&k) {
                            Some(v) => m.stack.push(*v),
                            None => m.stack.push(0.into()),
                        }
                    }
                    Yield::Store(k, v) => {
                        let account = self.state.get_mut(&ctx.target).unwrap();
                        account.storage.insert(k, v);
                    }
                    Yield::CalldataLoad(p) => {
                        let p: usize = p.low_u64() as usize;
                        m.stack.push(ctx.data[p..p + 32].into());
                    }
                    _ => unimplemented!(),
                },
                _ => break,
            }
        }
    }
}
