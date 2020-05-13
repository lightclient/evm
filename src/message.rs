use primitive_types::{H160, U256};

/// The message describing an EVM call, including a zero-depth calls from a transaction origin.
#[derive(Clone)]
pub enum Message {
    Call(Inner),
    DelegateCall(Inner),
    CallCode(Inner),
    Create(Inner),
    Create2(Inner, U256),
}

impl Message {
    pub fn inner(&self) -> &Inner {
        match self {
            Self::Call(i) => i,
            Self::DelegateCall(i) => i,
            Self::CallCode(i) => i,
            Self::Create(i) => i,
            Self::Create2(i, _) => i,
        }
    }
}

/// Inner value of a message.
#[derive(Clone)]
pub struct Inner {
    /// The amount of gas for message execution.
    pub gas: u64,

    /// The call depth.
    pub depth: u32,

    /// The destination of the message.
    pub target: H160,

    /// The sender of the message.
    pub caller: H160,

    /// The sender of the initial transaction.
    pub origin: H160,

    /// The amount of Ether transferred with the message.
    pub value: U256,

    /// The message input data.
    pub data: Vec<u8>,
}
