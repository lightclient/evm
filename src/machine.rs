use crate::env::Environment;
use crate::gas::*;
use crate::instructions::Op;
use crate::interupt::{Exit, Interupt, Yield};
use crate::message::Inner as Message;
use crate::utils::I256;

use log::{debug, error, trace};
use primitive_types::{H160, U256, U512};
use std::cmp::min;
use std::convert::TryInto;
use std::mem;
use std::ops::{BitAnd, BitOr, BitXor};

macro_rules! pop {
    ($s: expr) => {{
        match $s.pop() {
            Some(o) => o,
            None => return Interupt::Exit(Exit::StackUnderflow),
        }
    }};
}

macro_rules! push {
    ($s: expr, $v: expr) => {{
        $s.push($v.into())
    }};
}

macro_rules! from_base {
    ($base: expr, $op: expr) => {{
        ($op - $base) as usize
    }};
}

macro_rules! spend_gas {
    ($gas: expr, $amt: expr) => {{
        match $gas.checked_sub($amt) {
            Some(g) => {
                $gas = g;
            }
            None => return Interupt::Exit(Exit::OutOfGas),
        }
    }};
}

macro_rules! pay_mem_gas {
    ($m: expr, $begin: expr, $size: expr) => {{
        if $m.memory_size <= $begin + $size {
            let mut mem_expansion = ($begin + $size) - $m.memory_size;

            // round to closest word
            if mem_expansion % 32 != 0 {
                mem_expansion += 31
            }

            mem_expansion /= 32;

            spend_gas!(
                $m.msg.gas,
                G_MEMORY * mem_expansion as u64 + mem_expansion.pow(2) as u64 / 512
            );

            $m.memory_size += mem_expansion * 32;

            // resize memory if needed
            if $m.memory.len() < $begin + $size {
                $m.memory.resize($begin + $size + 256, 0);
            }
        }
    }};
}

macro_rules! set_mem {
    ($m: expr, $mem_begin: expr, $data_begin: expr, $data: expr, $size: expr) => {{
        pay_mem_gas!($m, $mem_begin, $size);

        // forgive me lord, i have sinned
        if $data.len() <= $data_begin {
            let data = vec![0; $size];
            $m.memory[$mem_begin..$mem_begin + $size].copy_from_slice(&data[0..$size]);
        } else if $data.len() <= $data_begin + $size {
            let mut data = vec![0; $size];
            data[0..$data.len() - $data_begin].copy_from_slice(&$data[$data_begin..$data.len()]);
            $m.memory[$mem_begin..$mem_begin + $size].copy_from_slice(&data[0..$size]);
        } else {
            $m.memory[$mem_begin..$mem_begin + $size]
                .copy_from_slice(&$data[$data_begin..$data_begin + $size]);
        }
    }};
}

pub struct Machine<'a> {
    pub pc: usize,
    pub stack: Vec<U256>,
    pub memory: Vec<u8>,
    pub memory_size: usize,
    pub code: &'a [u8],
    pub msg: Message,
    pub env: &'a Environment,
}

impl<'a> Machine<'a> {
    pub fn new(code: &'a [u8], msg: Message, env: &'a Environment) -> Self {
        Self {
            pc: 0,
            stack: vec![],
            memory: vec![0; 128],
            memory_size: 0,
            code,
            msg,
            env,
        }
    }

    pub fn run(&mut self) -> Interupt<Yield, Exit> {
        while self.pc < self.code.len() {
            trace!(
                "pc: {}, code[pc+1]: {:x?}, stack: {:x?}",
                self.pc,
                &self.code.get(self.pc + 1),
                self.stack,
            );

            let op: Op = unsafe { mem::transmute(self.code[self.pc]) };
            self.pc += 1;

            debug!("{:?}", op);

            match op {
                Op::Stop => return Interupt::Exit(Exit::Stop),
                Op::Add => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    let (r, _) = pop!(self.stack).overflowing_add(pop!(self.stack));
                    self.stack.push(r);
                }
                Op::Mul => {
                    spend_gas!(self.msg.gas, G_LOW);
                    let (r, _) = pop!(self.stack).overflowing_mul(pop!(self.stack));
                    self.stack.push(r);
                }
                Op::Sub => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    let (r, _) = pop!(self.stack).overflowing_sub(pop!(self.stack));
                    self.stack.push(r);
                }
                Op::Div => {
                    spend_gas!(self.msg.gas, G_LOW);
                    match pop!(self.stack).checked_div(pop!(self.stack)) {
                        Some(r) => self.stack.push(r),
                        None => self.stack.push(0.into()),
                    }
                }
                Op::Sdiv => {
                    spend_gas!(self.msg.gas, G_LOW);
                    let op1: I256 = pop!(self.stack).into();
                    let op2: I256 = pop!(self.stack).into();
                    self.stack.push((op1 / op2).into())
                }
                Op::Mod => {
                    spend_gas!(self.msg.gas, G_LOW);
                    let r = pop!(self.stack)
                        .checked_rem(pop!(self.stack))
                        .unwrap_or(0.into());
                    self.stack.push(r);
                }
                Op::Smod => {
                    spend_gas!(self.msg.gas, G_LOW);
                    let op1: I256 = pop!(self.stack).into();
                    let op2: I256 = pop!(self.stack).into();
                    let r: I256 = op1.checked_rem(op2).unwrap_or(I256::zero());

                    self.stack.push(r.into());
                }
                Op::Addmod => {
                    spend_gas!(self.msg.gas, G_MID);
                    let op1: U512 = pop!(self.stack).into();
                    let op2: U512 = pop!(self.stack).into();
                    let op3: U512 = pop!(self.stack).into();

                    let (mut r, _) = op1.overflowing_add(op2);
                    r = r.checked_rem(op3).unwrap_or(0.into());
                    let r: U256 = r
                        .try_into()
                        .expect("op3 is less than U256::max_value(), thus it never overflows");

                    self.stack.push(r);
                }
                Op::Mulmod => {
                    spend_gas!(self.msg.gas, G_MID);
                    let op1: U512 = pop!(self.stack).into();
                    let op2: U512 = pop!(self.stack).into();
                    let op3: U512 = pop!(self.stack).into();

                    let (mut r, _) = op1.overflowing_mul(op2);
                    r = r.checked_rem(op3).unwrap_or(0.into());
                    let r: U256 = r
                        .try_into()
                        .expect("op3 is less than U256::max_value(), thus it never overflows");

                    self.stack.push(r);
                }
                Op::Exp => {
                    // gas todo
                    let mut op1 = pop!(self.stack);
                    let mut op2 = pop!(self.stack);
                    let mut r: U256 = 1.into();

                    while op2 != 0.into() {
                        if op2 & 1.into() != 0.into() {
                            r = r.overflowing_mul(op1).0;
                        }
                        op2 = op2 >> 1;
                        op1 = op1.overflowing_mul(op1).0;
                    }

                    self.stack.push(r);
                }
                Op::Signextend => {
                    spend_gas!(self.msg.gas, G_LOW);
                    let op1 = pop!(self.stack);
                    let op2 = pop!(self.stack);
                    let mut ret = U256::zero();

                    if op1 > U256::from(32) {
                        ret = op2;
                    } else {
                        let len: usize = op1.as_usize();
                        let t: usize = 8 * (len + 1) - 1;
                        let t_bit_mask = U256::one() << t;
                        let t_value = (op2 & t_bit_mask) >> t;
                        for i in 0..256 {
                            let bit_mask = U256::one() << i;
                            let i_value = (op2 & bit_mask) >> i;
                            if i <= t {
                                ret = ret.overflowing_add(i_value << i).0;
                            } else {
                                ret = ret.overflowing_add(t_value << i).0;
                            }
                        }
                    }

                    self.stack.push(ret)
                }
                Op::Lt => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    if pop!(self.stack).lt(&pop!(self.stack)) {
                        self.stack.push(1.into());
                    } else {
                        self.stack.push(0.into())
                    }
                }
                Op::Gt => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    if pop!(self.stack).gt(&pop!(self.stack)) {
                        self.stack.push(1.into());
                    } else {
                        self.stack.push(0.into())
                    }
                }
                Op::Slt => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    let op1: I256 = pop!(self.stack).into();
                    let op2: I256 = pop!(self.stack).into();

                    if op1.lt(&op2) {
                        self.stack.push(1.into());
                    } else {
                        self.stack.push(0.into())
                    }
                }
                Op::Sgt => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    let op1: I256 = pop!(self.stack).into();
                    let op2: I256 = pop!(self.stack).into();

                    if op1.gt(&op2) {
                        self.stack.push(1.into());
                    } else {
                        self.stack.push(0.into())
                    }
                }
                Op::Eq => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    if pop!(self.stack).eq(&pop!(self.stack)) {
                        self.stack.push(1.into());
                    } else {
                        self.stack.push(0.into())
                    }
                }
                Op::Iszero => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    if pop!(self.stack) == U256::zero() {
                        self.stack.push(1.into());
                    } else {
                        self.stack.push(0.into());
                    }
                }
                Op::And => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    let r = pop!(self.stack).bitand(pop!(self.stack));
                    self.stack.push(r);
                }
                Op::Or => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    let r = pop!(self.stack).bitor(pop!(self.stack));
                    self.stack.push(r);
                }
                Op::Xor => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    let r = pop!(self.stack).bitxor(pop!(self.stack));
                    self.stack.push(r);
                }
                Op::Not => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    let r = !pop!(self.stack);
                    self.stack.push(r);
                }
                Op::Byte => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    let idx = pop!(self.stack).low_u64().checked_rem(32).unwrap_or(0);
                    let op: [u8; 32] = pop!(self.stack).into();
                    self.stack.push(op[idx as usize].into());
                }
                Op::Address => {
                    spend_gas!(self.msg.gas, G_BASE);
                    self.stack.push(self.msg.target.as_bytes().into())
                }
                Op::Balance => {}
                Op::Origin => {
                    spend_gas!(self.msg.gas, G_BASE);
                    self.stack.push(self.msg.origin.as_bytes().into());
                }
                Op::Caller => {
                    spend_gas!(self.msg.gas, G_BASE);
                    self.stack.push(self.msg.caller.as_bytes().into());
                }
                Op::CallValue => {
                    spend_gas!(self.msg.gas, G_BASE);
                    self.stack.push(self.msg.value);
                }
                Op::CalldataLoad => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    let begin = pop!(self.stack);
                    let mut ret = [0u8; 32];

                    if begin <= usize::max_value().into() {
                        let begin = begin.as_usize();

                        if begin < self.msg.data.len() {
                            let end = match begin.checked_add(32) {
                                Some(end) => min(end, self.msg.data.len()),
                                None => min(usize::max_value(), self.msg.data.len()),
                            };

                            trace!(
                                "Loading calldata[{}..{}], calldata len: {}",
                                begin,
                                end,
                                self.msg.data.len()
                            );

                            ret[0..(end - begin)].copy_from_slice(&self.msg.data[begin..end]);
                        }
                    }

                    self.stack.push(ret.into());
                }
                Op::CalldataSize => {
                    spend_gas!(self.msg.gas, G_BASE);
                    self.stack.push(self.msg.data.len().into());
                }
                Op::CalldataCopy => {
                    spend_gas!(self.msg.gas, G_VERYLOW);

                    let mem_begin = pop!(self.stack).low_u64() as usize;
                    let data_begin = pop!(self.stack).low_u64() as usize;
                    let len = pop!(self.stack).low_u64() as usize;

                    set_mem!(self, mem_begin, data_begin, self.msg.data, len);
                }
                Op::CodeSize => {
                    spend_gas!(self.msg.gas, G_BASE);
                    self.stack.push(self.code.len().into());
                }
                Op::CodeCopy => {
                    spend_gas!(self.msg.gas, G_VERYLOW);

                    let mem_begin = pop!(self.stack).low_u64() as usize;
                    let code_begin = pop!(self.stack).low_u64() as usize;
                    let len = pop!(self.stack).low_u64() as usize;

                    set_mem!(self, mem_begin, code_begin, self.code, len);
                }
                Op::GasPrice => {
                    spend_gas!(self.msg.gas, G_BASE);
                    self.stack.push(self.env.gas_price);
                }
                Op::Coinbase => {
                    spend_gas!(self.msg.gas, G_BASE);
                    self.stack.push(self.env.coinbase.as_bytes().into());
                }
                Op::Timestamp => {
                    spend_gas!(self.msg.gas, G_BASE);
                    self.stack.push(self.env.timestamp);
                }
                Op::Number => {
                    spend_gas!(self.msg.gas, G_BASE);
                    self.stack.push(self.env.block_number);
                }
                Op::Difficulty => {
                    spend_gas!(self.msg.gas, G_BASE);
                    self.stack.push(self.env.difficulty);
                }
                Op::GasLimit => {
                    spend_gas!(self.msg.gas, G_BASE);
                    self.stack.push(self.env.gas_limit);
                }
                Op::Pop => {
                    spend_gas!(self.msg.gas, G_BASE);
                    let _ = pop!(self.stack);
                }
                Op::MLoad => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    let idx = pop!(self.stack).low_u64() as usize;
                    let mut ret = [0; 32];

                    if idx < self.memory.len() {
                        let mem_len = self.memory.len();
                        ret.copy_from_slice(&self.memory[idx..min(idx + 32, mem_len)]);
                    }

                    self.stack.push(ret.into());
                }
                Op::MStore => {
                    spend_gas!(self.msg.gas, G_VERYLOW);

                    let mem_begin = pop!(self.stack).as_u64() as usize;
                    let value = pop!(self.stack);

                    set_mem!(self, mem_begin, 0, <[u8; 32]>::from(value), 32);
                }
                Op::MStore8 => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    let mem_begin = pop!(self.stack).as_u64() as usize;
                    let value = pop!(self.stack);

                    set_mem!(self, mem_begin, 0, <[u8; 32]>::from(value)[31..32], 1);
                }
                Op::SLoad => {
                    spend_gas!(self.msg.gas, 200);
                    return Interupt::Yield(Yield::Load(pop!(self.stack)));
                }
                Op::SStore => {
                    // todo gas
                    return Interupt::Yield(Yield::Store(pop!(self.stack), pop!(self.stack)));
                }
                Op::Jump => {
                    spend_gas!(self.msg.gas, G_MID);
                    let dest = pop!(self.stack).low_u64() as usize;
                    match self
                        .code
                        .get(dest)
                        .and_then(|op| Some(unsafe { mem::transmute::<&u8, &Op>(op) }))
                    {
                        Some(Op::Jumpdest) => (),
                        _ => return Interupt::Exit(Exit::BadJump),
                    }

                    self.pc = dest;
                }
                Op::Jumpi => {
                    spend_gas!(self.msg.gas, G_HIGH);
                    let dest = pop!(self.stack).low_u64() as usize;
                    let condition = pop!(self.stack);

                    if condition == U256::one() {
                        match self
                            .code
                            .get(dest)
                            .and_then(|op| Some(unsafe { mem::transmute::<&u8, &Op>(op) }))
                        {
                            Some(Op::Jumpdest) => (),
                            _ => return Interupt::Exit(Exit::BadJump),
                        }

                        self.pc = dest;
                    }
                }
                Op::Pc => {
                    spend_gas!(self.msg.gas, G_BASE);
                    self.stack.push((self.pc - 1).into());
                }
                Op::MSize => {
                    spend_gas!(self.msg.gas, G_BASE);
                    self.stack.push(self.memory_size.into());
                }
                Op::Gas => {
                    spend_gas!(self.msg.gas, G_BASE);
                    self.stack.push(self.msg.gas.into());
                }
                Op::Jumpdest => spend_gas!(self.msg.gas, 1),
                Op::Push1
                | Op::Push2
                | Op::Push3
                | Op::Push4
                | Op::Push5
                | Op::Push6
                | Op::Push7
                | Op::Push8
                | Op::Push9
                | Op::Push10
                | Op::Push11
                | Op::Push12
                | Op::Push13
                | Op::Push14
                | Op::Push15
                | Op::Push16
                | Op::Push17
                | Op::Push18
                | Op::Push19
                | Op::Push20
                | Op::Push21
                | Op::Push22
                | Op::Push23
                | Op::Push24
                | Op::Push25
                | Op::Push26
                | Op::Push27
                | Op::Push28
                | Op::Push29
                | Op::Push30
                | Op::Push31
                | Op::Push32 => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    let base = from_base!(0x60, unsafe { mem::transmute::<Op, u8>(op) });
                    if self.pc + base < self.code.len() {
                        let o = &self.code[self.pc..self.pc + base + 1];
                        push!(self.stack, o);
                    } else {
                        return Interupt::Exit(Exit::StackUnderflow);
                    }

                    self.pc += base + 1;
                }
                Op::Dup1
                | Op::Dup2
                | Op::Dup3
                | Op::Dup4
                | Op::Dup5
                | Op::Dup6
                | Op::Dup7
                | Op::Dup8
                | Op::Dup9
                | Op::Dup10
                | Op::Dup11
                | Op::Dup12
                | Op::Dup13
                | Op::Dup14
                | Op::Dup15
                | Op::Dup16 => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    let dup_idx = from_base!(0x80, unsafe { mem::transmute::<Op, u8>(op) });

                    if !self.stack.is_empty() && dup_idx < self.stack.len() {
                        let idx = self.stack.len() - dup_idx - 1;
                        let e = self.stack[idx];
                        self.stack.push(e);
                    } else {
                        return Interupt::Exit(Exit::StackUnderflow);
                    }
                }
                Op::Swap1
                | Op::Swap2
                | Op::Swap3
                | Op::Swap4
                | Op::Swap5
                | Op::Swap6
                | Op::Swap7
                | Op::Swap8
                | Op::Swap9
                | Op::Swap10
                | Op::Swap11
                | Op::Swap12
                | Op::Swap13
                | Op::Swap14
                | Op::Swap15
                | Op::Swap16 => {
                    spend_gas!(self.msg.gas, G_VERYLOW);
                    let swap_idx = from_base!(0x90, unsafe { mem::transmute::<Op, u8>(op) });

                    if 2 <= self.stack.len() && swap_idx < self.stack.len() - 1 {
                        let top = self.stack.len() - 1;
                        let idx = top - swap_idx - 1;
                        self.stack.swap(top, idx);
                    } else {
                        return Interupt::Exit(Exit::StackUnderflow);
                    }
                }

                Op::Return => return Interupt::Exit(Exit::Ret(pop!(self.stack), pop!(self.stack))),
                Op::Revert => {
                    return Interupt::Exit(Exit::Revert(pop!(self.stack), pop!(self.stack)))
                }
                Op::SelfDestruct => {
                    let val = pop!(self.stack);

                    // bad
                    let mut bytes = [0; 32];
                    val.to_big_endian(&mut bytes);
                    let mut address = [0; 20];
                    address.copy_from_slice(&bytes[12..32]);

                    return Interupt::Exit(Exit::SelfDestruct(H160::from(&address)));
                }
                _ => {
                    error!("UNSUPPORTED OP: {:?}", op);
                    return Interupt::Exit(Exit::NotSupported);
                }
            }
        }

        Interupt::Exit(Exit::Ret(0.into(), 0.into()))
    }
}
