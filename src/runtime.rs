use crate::account::Account;
use crate::ctx::Context;
use crate::env::Environment;
use crate::interupt::{Exit, Interupt, Yield};
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
                Interupt::Exit(Exit::SelfDestruct(target)) => {
                    let account = self.state.remove(&ctx.target).unwrap();
                    self.state.remove(&ctx.target);
                    let raw_target: [u8; 32] = target.into();
                    let target = self
                        .state
                        .entry(H160::from_slice(&raw_target[12..32]))
                        .or_default();

                    target.balance += account.balance;
                }
                _ => break,
            }
        }
    }
}
