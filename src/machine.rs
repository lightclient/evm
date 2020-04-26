use crate::ctx::Context;
use crate::env::Environment;
use crate::instructions::*;
use crate::interupt::{Exit, Interupt, Yield};

use primitive_types::U256;

macro_rules! pop {
    ($s: expr) => {{
        $s.pop().unwrap()
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
                    let r = pop!(self.stack) + pop!(self.stack);
                    self.stack.push(r);
                }
                SUB => {
                    let r = pop!(self.stack) - pop!(self.stack);
                    self.stack.push(r);
                }
                MUL => {
                    let r = pop!(self.stack) * pop!(self.stack);
                    self.stack.push(r);
                }
                DIV => {
                    let r = pop!(self.stack) / pop!(self.stack);
                    self.stack.push(r);
                }
                op @ PUSH1..=PUSH32 => {
                    self.pc += 1;
                    let o = &self.code[self.pc..self.pc + from_base!(PUSH1, op) + 1];
                    push!(self.stack, o);
                }
                op @ DUP1..=DUP16 => {
                    let len = self.stack.len() - 1;
                    let idx = len - from_base!(DUP1, op);
                    push!(self.stack, self.stack.get(idx).unwrap());
                }
                op @ SWAP1..=SWAP16 => {
                    let len = self.stack.len() - 1;
                    let idx = len - from_base!(SWAP1, op);
                    self.stack.swap(len, idx);
                }
                _ => unimplemented!(),
            }

            self.pc += 1;
        }

        Interupt::Exit(Exit::Ret)
    }
}
