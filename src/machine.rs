use crate::ctx::Context;
use crate::env::Environment;
use crate::instructions::*;
use crate::interupt::{Exit, Interupt, Yield};
use crate::utils::I256;

use primitive_types::{U256, U512};
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
            memory: vec![],
            code,
            ctx,
            env,
        }
    }

    pub fn run(&mut self) -> Interupt<Yield, Exit> {
        while self.pc < self.code.len() {
            let op = self.code[self.pc];
            self.pc += 1;

            match op {
                STOP => {
                    return Interupt::Exit(Exit::Stop);
                }
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
                    let len = self.stack.len() - 1;
                    let idx = len - from_base!(DUP1, op);

                    match self.stack.get(idx).map(|e| *e) {
                        Some(o) => self.stack.push(o),
                        None => return Interupt::Exit(Exit::StackUnderflow),
                    }
                }
                op @ SWAP1..=SWAP16 => {
                    let len = self.stack.len() - 1;
                    let idx = len - from_base!(SWAP1, op);
                    self.stack.swap(len, idx);
                }
                SLOAD => {
                    return Interupt::Yield(Yield::Load(pop!(self.stack)));
                }
                SSTORE => {
                    return Interupt::Yield(Yield::Store(pop!(self.stack), pop!(self.stack)));
                }
                CALLDATALOAD => {
                    return Interupt::Yield(Yield::CalldataLoad(pop!(self.stack)));
                }
                op => {
                    eprintln!("UNSUPPORTED OP: {:x}", op);
                    return Interupt::Exit(Exit::NotSupported);
                }
            }
        }

        Interupt::Exit(Exit::Ret)
    }
}
