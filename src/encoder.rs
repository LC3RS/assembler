use num_traits::ToPrimitive;
use crate::enums::{OpCode, Register, Token};
use crate::utils::sign_extend;

pub fn encode_fill() -> Vec<u16> {
    vec![0x00]
}

pub fn encode_blkw(c: &u16) -> Vec<u16> {
    vec![0x00; *c as usize]
}

pub fn encode_stringz(s: &str) -> Vec<u16> {
    let mut bin: Vec<_> = s.bytes().map(|e| e as u16).collect();
    bin.push(0); // Null-terminated strings
    bin
}

pub fn encode_orig(origin: &u16) -> Vec<u16> {
    vec![*origin]
}

pub fn encode_br(t:&Token,offset: u16) -> Vec<u16> {
    let mut com = 0u16;
    if let Token::Op(op ) = t {
        let fin_op =  op.to_u16().unwrap();
        com = com | (fin_op << 9);
    }
    com = com | sign_extend(offset, 9);
    vec![com]
}

pub fn encode_add(dr:&Register, sr1:&Register, sr2:u16) -> Vec<u16> {
    let com = (dr.to_u16().unwrap() << 9 ) | (sr1.to_u16().unwrap() << 6 ) | 0x1000 | sr2;
    vec![com]
}

pub fn encode_and(dr:&Register, sr1:&Register, sr2:u16) -> Vec<u16> {
    let com = (dr.to_u16().unwrap() << 9 ) | (sr1.to_u16().unwrap() << 6 ) | 0x5000 | sr2;
    vec![com]
}