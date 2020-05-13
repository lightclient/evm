use crate::env::Environment;
use crate::host::Host;
use crate::interupt::{Exit, Interupt, Yield};
use crate::machine::Machine;
use crate::message::Message;

use log::info;

pub fn execute<T: Host>(host: &mut T, env: &Environment, msg: Message, code: &[u8]) -> Exit {
    let mut m = Machine::new(code, msg.inner().clone(), env);

    loop {
        let i = m.run();

        info!("interupt reason: {:?}", i);

        match i {
            Interupt::Yield(y) => match y {
                Yield::Load(k) => {
                    let item = host.get_storage(&msg.inner().target, &k);
                    m.stack.push(item);
                }
                Yield::Store(k, v) => {
                    host.set_storage(&msg.inner().target, k, v);
                }
                _ => unimplemented!(),
            },
            Interupt::Exit(Exit::SelfDestruct(beneficiary)) => {
                host.self_destruct(&msg.inner().target, beneficiary);
                return Exit::SelfDestruct(beneficiary);
            }
            Interupt::Exit(e) => return e,
        }
    }
}
