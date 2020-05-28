use evmc_vm::{ffi::evmc_status_code as StatusCode, ExecutionResult as Result};
use primitive_types::{H160, U256};

#[derive(Debug)]
pub enum Interupt<Y, E> {
    Yield(Y),
    Exit(E),
}

#[derive(Debug, PartialEq)]
pub enum Yield {
    // external
    Call,
    Create,
    Store(U256, U256),
    Load(U256),
}

#[derive(Debug, PartialEq)]
pub enum Exit {
    // successful
    Stop,
    Ret(U256, U256),
    SelfDestruct(H160),

    // normal error
    StackUnderflow,
    StackOverflow,
    BadJump,
    BadRange,
    InvalidOp,
    CallOverflow,
    OutOfGas,

    // revert
    Revert(U256, U256),

    // fatal
    NotSupported,
}

impl Exit {
    pub fn to_result(self, gas: i64, mem: &[u8]) -> Result {
        match self {
            Self::Stop => Result::success(gas, None),
            Self::Ret(offset, len) => {
                let begin = offset.low_u64() as usize;
                let end = begin + len.low_u64() as usize;

                Result::success(gas, Some(&mem[begin..end]))
            }
            Self::SelfDestruct(_) => unreachable!(),
            Self::StackUnderflow => Result::new(StatusCode::EVMC_STACK_UNDERFLOW, 0, None),
            Self::StackOverflow => Result::new(StatusCode::EVMC_STACK_OVERFLOW, 0, None),
            Self::BadJump => Result::new(StatusCode::EVMC_BAD_JUMP_DESTINATION, 0, None),
            Self::BadRange => Result::new(StatusCode::EVMC_INVALID_MEMORY_ACCESS, 0, None),
            Self::InvalidOp => Result::new(StatusCode::EVMC_INVALID_INSTRUCTION, 0, None),
            Self::CallOverflow => Result::new(StatusCode::EVMC_CALL_DEPTH_EXCEEDED, 0, None),
            Self::OutOfGas => Result::new(StatusCode::EVMC_OUT_OF_GAS, 0, None),
            Self::Revert(offset, len) => {
                let begin = offset.low_u64() as usize;
                let end = begin + len.low_u64() as usize;

                Result::new(StatusCode::EVMC_REVERT, gas, Some(&mem[begin..end]))
            }
            Self::NotSupported => Result::new(StatusCode::EVMC_UNDEFINED_INSTRUCTION, 0, None),
        }
    }
}
