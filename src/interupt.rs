pub enum Interupt<Y, E> {
    Yield(Y),
    Exit(E),
}

#[derive(Debug, PartialEq)]
pub enum Yield {
    // intrinsic
    Stop,
    Revert,
    Ret,

    // external
    Call,
    Create,
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
