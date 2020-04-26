use crate::ctx::Context;
use crate::env::Environment;
use crate::instructions::*;
use crate::interupt::{Exit, Interupt, Yield};

use primitive_types::U256;

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
                op @ PUSH1..=PUSH32 => {
                    self.pc += 1;

                    if self.pc + from_base!(PUSH1, op) < self.code.len() {
                        let o = &self.code[self.pc..self.pc + from_base!(PUSH1, op) + 1];
                        push!(self.stack, o);
                    } else {
                        return Interupt::Exit(Exit::StackUnderflow);
                    }
                }
                op @ DUP1..=DUP16 => {
                    let len = self.stack.len() - 1;
                    let idx = len - from_base!(DUP1, op);
                    match self.stack.get(idx) {
                        Some(o) => push!(self.stack, o),
                        None => return Interupt::Exit(Exit::StackUnderflow),
                    }
                }
                op @ SWAP1..=SWAP16 => {
                    let len = self.stack.len() - 1;
                    let idx = len - from_base!(SWAP1, op);
                    self.stack.swap(len, idx);
                }
                _ => return Interupt::Exit(Exit::NotSupported),
            }

            self.pc += 1;
        }

        Interupt::Exit(Exit::Ret)
    }
}
