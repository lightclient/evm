use primitive_types::U256;

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
}

#[derive(Debug, PartialEq)]
pub enum Exit {
    // successful
    Stop,
    Ret,
    SelfDestruct,

    // normal error
    StackUnderflow,
    StackOverflow,
    BadJump,
    BadRange,
    InvalidOp,
    CallOverflow,
    OutOfGas,

    // revert
    Revert,

    // fatal
    NotSupported,
    UnhandledInterrupt,
}
