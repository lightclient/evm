pub const STOP: u8 = 0x00;
pub const ADD: u8 = 0x01;
pub const MUL: u8 = 0x02;
pub const SUB: u8 = 0x03;
pub const DIV: u8 = 0x04;
pub const SDIV: u8 = 0x05;
pub const MOD: u8 = 0x06;
pub const SMOD: u8 = 0x07;
pub const ADDMOD: u8 = 0x08;
pub const MULMOD: u8 = 0x09;
pub const EXP: u8 = 0x0A;
pub const SIGEXTEND: u8 = 0x0B;

pub const LT: u8 = 0x10;
pub const GT: u8 = 0x11;
pub const SLT: u8 = 0x12;
pub const SGT: u8 = 0x13;
pub const EQ: u8 = 0x14;
pub const ISZERO: u8 = 0x15;
pub const AND: u8 = 0x16;
pub const OR: u8 = 0x17;
pub const XOR: u8 = 0x18;
pub const NOT: u8 = 0x19;
pub const BYTE: u8 = 0x1A;
pub const SHL: u8 = 0x1B;
pub const SHR: u8 = 0x1C;
pub const SAR: u8 = 0x1D;

pub const SHA3: u8 = 0x20;

pub const CALLDATALOAD: u8 = 0x35;

pub const POP: u8 = 0x50;

pub const SLOAD: u8 = 0x54;
pub const SSTORE: u8 = 0x55;

pub const PUSH1: u8 = 0x60;
pub const PUSH2: u8 = 0x61;
pub const PUSH3: u8 = 0x62;
pub const PUSH4: u8 = 0x63;
pub const PUSH5: u8 = 0x64;
pub const PUSH6: u8 = 0x65;
pub const PUSH7: u8 = 0x66;
pub const PUSH8: u8 = 0x67;
pub const PUSH9: u8 = 0x68;
pub const PUSH10: u8 = 0x69;
pub const PUSH11: u8 = 0x6A;
pub const PUSH12: u8 = 0x6B;
pub const PUSH13: u8 = 0x6C;
pub const PUSH14: u8 = 0x6D;
pub const PUSH15: u8 = 0x6E;
pub const PUSH16: u8 = 0x6F;
pub const PUSH17: u8 = 0x70;
pub const PUSH18: u8 = 0x71;
pub const PUSH19: u8 = 0x72;
pub const PUSH20: u8 = 0x73;
pub const PUSH21: u8 = 0x74;
pub const PUSH22: u8 = 0x75;
pub const PUSH23: u8 = 0x76;
pub const PUSH24: u8 = 0x77;
pub const PUSH25: u8 = 0x78;
pub const PUSH26: u8 = 0x79;
pub const PUSH27: u8 = 0x7A;
pub const PUSH28: u8 = 0x7B;
pub const PUSH29: u8 = 0x7C;
pub const PUSH30: u8 = 0x7D;
pub const PUSH31: u8 = 0x7E;
pub const PUSH32: u8 = 0x7F;

pub const DUP1: u8 = 0x80;
pub const DUP2: u8 = 0x81;
pub const DUP3: u8 = 0x82;
pub const DUP4: u8 = 0x83;
pub const DUP5: u8 = 0x84;
pub const DUP6: u8 = 0x85;
pub const DUP7: u8 = 0x86;
pub const DUP8: u8 = 0x87;
pub const DUP9: u8 = 0x88;
pub const DUP10: u8 = 0x89;
pub const DUP11: u8 = 0x8A;
pub const DUP12: u8 = 0x8B;
pub const DUP13: u8 = 0x8C;
pub const DUP14: u8 = 0x8D;
pub const DUP15: u8 = 0x8E;
pub const DUP16: u8 = 0x8F;

pub const SWAP1: u8 = 0x90;
pub const SWAP2: u8 = 0x91;
pub const SWAP3: u8 = 0x92;
pub const SWAP4: u8 = 0x93;
pub const SWAP5: u8 = 0x94;
pub const SWAP6: u8 = 0x95;
pub const SWAP7: u8 = 0x96;
pub const SWAP8: u8 = 0x97;
pub const SWAP9: u8 = 0x98;
pub const SWAP10: u8 = 0x99;
pub const SWAP11: u8 = 0x9A;
pub const SWAP12: u8 = 0x9B;
pub const SWAP13: u8 = 0x9C;
pub const SWAP14: u8 = 0x9D;
pub const SWAP15: u8 = 0x9E;
pub const SWAP16: u8 = 0x9F;

pub const RETURN: u8 = 0xF3;
pub const REVERT: u8 = 0xFD;
pub const INVALID: u8 = 0xFE;
pub const SELFDESTRUCT: u8 = 0xFF;

pub fn op_to_str(op: u8) -> &'static str {
    match op {
        STOP => "STOP",
        ADD => "ADD",
        MUL => "MUL",
        SUB => "SUB",
        DIV => "DIV",
        SDIV => "SDIV",
        MOD => "MOD",
        SMOD => "SMOD",
        ADDMOD => "ADDMOD",
        MULMOD => "MULMOD",
        EXP => "EXP",
        SIGEXTEND => "SIGEXTEND",
        LT => "LT",
        GT => "GT",
        SLT => "SLT",
        SGT => "SGT",
        EQ => "EQ",
        ISZERO => "ISZERO",
        AND => "AND",
        OR => "OR",
        XOR => "XOR",
        NOT => "NOT",
        BYTE => "BYTE",
        SHL => "SHL",
        SHR => "SHR",
        SAR => "SAR",
        SHA3 => "SHA3",
        CALLDATALOAD => "CALLDATALOAD",
        POP => "POP",
        SLOAD => "SLOAD",
        SSTORE => "SSTORE",
        PUSH1..=PUSH32 => "PUSH",
        DUP1..=DUP16 => "DUP",
        SWAP1..=SWAP16 => "DUP",
        RETURN => "RETURN",
        REVERT => "REVERT",
        INVALID => "INVALID",
        SELFDESTRUCT => "SELFDESTRUCT",

        _ => "UNKNOWN",
    }
}