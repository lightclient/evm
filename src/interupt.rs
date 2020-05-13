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
    UnhandledInterrupt,
}
