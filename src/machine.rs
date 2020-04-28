use crate::ctx::Context;
use crate::env::Environment;
use crate::instructions::*;
use crate::interupt::{Exit, Interupt, Yield};
use crate::utils::I256;

use log::{debug, error, info, trace, warn};
use primitive_types::{U256, U512};
use std::cmp::min;
use std::convert::TryInto;
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

pub struct Machine<'a> {
    pub pc: usize,
    pub stack: Vec<U256>,
    pub memory: Vec<u8>,
    pub code: Vec<u8>,
    pub ctx: Context,
    pub env: &'a Environment,
}

impl<'a> Machine<'a> {
    pub fn new(code: Vec<u8>, ctx: Context, env: &'a Environment) -> Self {
        Self {
            pc: 0,
            stack: vec![],
            memory: vec![0; 1024],
            code,
            ctx,
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

            let op = self.code[self.pc];
            self.pc += 1;

            debug!("{}({:x})", op_to_str(op), op);

            match op {
                STOP => return Interupt::Exit(Exit::Stop),
                ADD => {
                    let (r, _) = pop!(self.stack).overflowing_add(pop!(self.stack));
                    self.stack.push(r);
                }
                SUB => {
                    let (r, _) = pop!(self.stack).overflowing_sub(pop!(self.stack));
                    self.stack.push(r);
                }
                MUL => {
                    let (r, _) = pop!(self.stack).overflowing_mul(pop!(self.stack));
                    self.stack.push(r);
                }
                DIV => match pop!(self.stack).checked_div(pop!(self.stack)) {
                    Some(r) => self.stack.push(r),
                    None => self.stack.push(0.into()),
                },
                SDIV => {
                    let op1: I256 = pop!(self.stack).into();
                    let op2: I256 = pop!(self.stack).into();
                    self.stack.push((op1 / op2).into())
                }
                MOD => {
                    let r = pop!(self.stack)
                        .checked_rem(pop!(self.stack))
                        .unwrap_or(0.into());
                    self.stack.push(r);
                }
                SMOD => {
                    let op1: I256 = pop!(self.stack).into();
                    let op2: I256 = pop!(self.stack).into();
                    let r: I256 = op1.checked_rem(op2).unwrap_or(I256::zero());

                    self.stack.push(r.into());
                }
                ADDMOD => {
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
                MULMOD => {
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
                EXP => {
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
                SIGEXTEND => {
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
                LT => {
                    if pop!(self.stack).lt(&pop!(self.stack)) {
                        self.stack.push(1.into());
                    } else {
                        self.stack.push(0.into())
                    }
                }
                SLT => {
                    let op1: I256 = pop!(self.stack).into();
                    let op2: I256 = pop!(self.stack).into();

                    if op1.lt(&op2) {
                        self.stack.push(1.into());
                    } else {
                        self.stack.push(0.into())
                    }
                }
                GT => {
                    if pop!(self.stack).gt(&pop!(self.stack)) {
                        self.stack.push(1.into());
                    } else {
                        self.stack.push(0.into())
                    }
                }
                SGT => {
                    let op1: I256 = pop!(self.stack).into();
                    let op2: I256 = pop!(self.stack).into();

                    if op1.gt(&op2) {
                        self.stack.push(1.into());
                    } else {
                        self.stack.push(0.into())
                    }
                }
                EQ => {
                    if pop!(self.stack).eq(&pop!(self.stack)) {
                        self.stack.push(1.into());
                    } else {
                        self.stack.push(0.into())
                    }
                }
                ISZERO => {
                    if pop!(self.stack) == U256::zero() {
                        self.stack.push(1.into());
                    } else {
                        self.stack.push(0.into());
                    }
                }
                AND => {
                    let r = pop!(self.stack).bitand(pop!(self.stack));
                    self.stack.push(r);
                }
                OR => {
                    let r = pop!(self.stack).bitor(pop!(self.stack));
                    self.stack.push(r);
                }
                XOR => {
                    let r = pop!(self.stack).bitxor(pop!(self.stack));
                    self.stack.push(r);
                }
                NOT => {
                    let r = !pop!(self.stack);
                    self.stack.push(r);
                }
                BYTE => {
                    let idx = pop!(self.stack).low_u64().checked_rem(32).unwrap_or(0);
                    let op: [u8; 32] = pop!(self.stack).into();
                    self.stack.push(op[idx as usize].into());
                }
                TIMESTAMP => self.stack.push(self.env.timestamp),
                COINBASE => self.stack.push(self.env.coinbase.as_bytes().into()),
                NUMBER => self.stack.push(self.env.block_number),
                DIFFICULTY => self.stack.push(self.env.difficulty),
                GASLIMIT => self.stack.push(self.env.gas_limit),
                POP => {
                    let _ = pop!(self.stack);
                }
                MLOAD => {
                    let idx = pop!(self.stack).low_u64() as usize;
                    let mut ret = [0; 32];

                    if idx < self.memory.len() {
                        let mem_len = self.memory.len();
                        ret.copy_from_slice(&self.memory[idx..min(idx + 32, mem_len)]);
                    }

                    self.stack.push(ret.into());
                }
                op @ PUSH1..=PUSH32 => {
                    if self.pc + from_base!(PUSH1, op) < self.code.len() {
                        let o = &self.code[self.pc..self.pc + from_base!(PUSH1, op) + 1];
                        push!(self.stack, o);
                    } else {
                        return Interupt::Exit(Exit::StackUnderflow);
                    }

                    self.pc += from_base!(PUSH1, op) + 1;
                }
                op @ DUP1..=DUP16 => {
                    let dup_idx = from_base!(DUP1, op);

                    if !self.stack.is_empty() && dup_idx < self.stack.len() {
                        let idx = self.stack.len() - from_base!(DUP1, op) - 1;
                        let e = self.stack[idx];
                        self.stack.push(e);
                    } else {
                        return Interupt::Exit(Exit::StackUnderflow);
                    }
                }
                op @ SWAP1..=SWAP16 => {
                    let swap_idx = from_base!(SWAP1, op) + 1;

                    if !self.stack.is_empty() && swap_idx < self.stack.len() {
                        let top = self.stack.len() - 1;
                        let idx = top - swap_idx;
                        self.stack.swap(top, idx);
                    } else {
                        return Interupt::Exit(Exit::StackUnderflow);
                    }
                }
                SLOAD => return Interupt::Yield(Yield::Load(pop!(self.stack))),
                SSTORE => return Interupt::Yield(Yield::Store(pop!(self.stack), pop!(self.stack))),
                ADDRESS => self.stack.push(self.ctx.target.as_bytes().into()),
                BALANCE => {}
                ORIGIN => self.stack.push(self.ctx.origin.as_bytes().into()),
                CALLER => self.stack.push(self.ctx.caller.as_bytes().into()),
                CALLVALUE => self.stack.push(self.ctx.value),
                CALLDATALOAD => {
                    let begin = pop!(self.stack);
                    let mut ret = [0u8; 32];

                    if begin <= usize::max_value().into() {
                        let begin = begin.as_usize();

                        if begin < self.ctx.data.len() {
                            let end = match begin.checked_add(32) {
                                Some(end) => min(end, self.ctx.data.len()),
                                None => min(usize::max_value(), self.ctx.data.len()),
                            };

                            trace!(
                                "Loading calldata[{}..{}], calldata len: {}",
                                begin,
                                end,
                                self.ctx.data.len()
                            );

                            ret[0..(end - begin)].copy_from_slice(&self.ctx.data[begin..end]);
                        }
                    }

                    self.stack.push(ret.into());
                }
                CALLDATASIZE => self.stack.push(self.ctx.data.len().into()),
                CALLDATACOPY => {
                    let mem_begin = pop!(self.stack).low_u64() as usize;
                    let data_begin = pop!(self.stack).low_u64() as usize;
                    let len = pop!(self.stack).low_u64() as usize;

                    if self.ctx.data.len() < data_begin {
                        self.stack.push(0.into());
                    } else {
                        let (mem_end, _) = mem_begin.overflowing_add(len);

                        if self.memory.len() < mem_end {
                            self.memory.resize(1024, 0);
                        }

                        let (data_end, f) = data_begin.overflowing_add(len);
                        let data_end = if f { self.ctx.data.len() } else { data_end };

                        self.memory[mem_begin..mem_end]
                            .copy_from_slice(&self.ctx.data[data_begin..data_end]);
                    }
                }
                CODESIZE => self.stack.push(self.code.len().into()),
                CODECOPY => {
                    let mem_begin = pop!(self.stack).low_u64() as usize;
                    let code_begin = pop!(self.stack).low_u64() as usize;
                    let len = pop!(self.stack).low_u64() as usize;

                    if self.ctx.data.len() < code_begin {
                        self.stack.push(0.into());
                    } else {
                        let (mem_end, _) = mem_begin.overflowing_add(len);

                        if self.memory.len() < mem_end {
                            self.memory.resize(1024, 0);
                        }

                        let (code_end, f) = code_begin.overflowing_add(len);
                        let code_end = if f { self.code.len() } else { code_end };

                        self.memory[mem_begin..mem_end]
                            .copy_from_slice(&self.code[code_begin..code_end]);
                    }
                }
                GASPRICE => self.stack.push(self.env.gas_price),
                SELFDESTRUCT => return Interupt::Exit(Exit::SelfDestruct(pop!(self.stack))),
                op => {
                    error!("UNSUPPORTED OP: {}({:x})", op_to_str(op), op);
                    return Interupt::Exit(Exit::NotSupported);
                }
            }
        }

        Interupt::Exit(Exit::Ret)
    }
}
