use crate::env::Environment;
use crate::interupt::{Exit, Interupt, Yield};
use crate::machine::Machine;
use crate::message::{Kind as MessageKind, Message};

use evmc_declare::evmc_declare_vm;
use evmc_vm::{EvmcVm, ExecutionContext, ExecutionResult, Revision};
use log::info;
use primitive_types::U256;

#[evmc_declare_vm("fast_evm", "evm", "6.3.0-dev")]
pub struct Vm;

impl EvmcVm for Vm {
    fn init() -> Self {
        Vm {}
    }

    fn execute<'a>(
        &self,
        _revision: Revision,
        code: &'a [u8],
        msg: &'a Message,
        context: Option<&'a mut ExecutionContext<'a>>,
    ) -> ExecutionResult {
        if context.is_none() {
            return ExecutionResult::failure();
        }

        let context = context.unwrap();

        if msg.kind() != MessageKind::EVMC_CALL {
            return ExecutionResult::failure();
        }

        if code.len() == 0 {
            return ExecutionResult::success(msg.gas(), None);
        }

        let tx_context = context.get_tx_context().clone();
        let env = Environment::from(tx_context.clone());

        let mut m = Machine::new(code, msg, &env);

        loop {
            let i = m.run();

            info!("interupt reason: {:?}", i);

            match i {
                Interupt::Yield(y) => match y {
                    Yield::Load(k) => {
                        let mut raw = [0; 32];
                        k.to_big_endian(&mut raw);

                        let item = context.get_storage(
                            msg.destination(),
                            &evmc_vm::ffi::evmc_bytes32 { bytes: raw },
                        );

                        m.stack.push(U256::from_big_endian(&item.bytes));
                    }
                    Yield::Store(k, v) => {
                        let mut k_raw = [0; 32];
                        k.to_big_endian(&mut k_raw);

                        let mut v_raw = [0; 32];
                        v.to_big_endian(&mut v_raw);

                        context.set_storage(
                            msg.destination(),
                            &evmc_vm::ffi::evmc_bytes32 { bytes: k_raw },
                            &evmc_vm::ffi::evmc_bytes32 { bytes: v_raw },
                        );
                    }
                    _ => unimplemented!(),
                },
                Interupt::Exit(Exit::SelfDestruct(beneficiary)) => {
                    context.selfdestruct(
                        msg.destination(),
                        &evmc_vm::ffi::evmc_address {
                            bytes: beneficiary.to_fixed_bytes(),
                        },
                    );

                    return ExecutionResult::success(m.gas as i64, None);
                }
                Interupt::Exit(e) => return e.to_result(m.gas as i64, &m.memory),
            }
        }
    }
}
