use crate::enums::{Register, Token};
use num_traits::ToPrimitive;

pub fn encode_fill(v: u16) -> Vec<u16> {
    vec![v]
}

pub fn encode_blkw(c: u16) -> Vec<u16> {
    vec![0x00; c as usize]
}

pub fn encode_stringz(s: String) -> Vec<u16> {
    let mut bin: Vec<_> = s.bytes().map(|e| e as u16).collect();
    bin.push(0); // Null-terminated strings
    bin
}

pub fn encode_orig(origin: u16) -> Vec<u16> {
    vec![origin]
}

pub fn encode_br(t: &Token, offset: u16) -> Vec<u16> {
    let mut com = 0u16;
    if let Token::Op(op) = t {
        let fin_op = op.to_u16().unwrap();
        com |= fin_op << 9;
    }
    com |= offset;
    vec![com]
}

pub fn encode_add_reg(dr: Register, sr1: Register, sr2: Register) -> Vec<u16> {
    let com =
        (dr.to_u16().unwrap() << 9) | (sr1.to_u16().unwrap() << 6) | 0x1000 | sr2.to_u16().unwrap();
    vec![com]
}

pub fn encode_add_imm(dr: Register, sr1: Register, imm: u16) -> Vec<u16> {
    let com = (dr.to_u16().unwrap() << 9) | (sr1.to_u16().unwrap() << 6) | 0x1000 | imm | (1 << 5);
    vec![com]
}

pub fn encode_and_reg(dr: Register, sr1: Register, sr2: Register) -> Vec<u16> {
    let com =
        (dr.to_u16().unwrap() << 9) | (sr1.to_u16().unwrap() << 6) | 0x5000 | sr2.to_u16().unwrap();
    vec![com]
}

pub fn encode_and_imm(dr: Register, sr1: Register, imm: u16) -> Vec<u16> {
    let com = (dr.to_u16().unwrap() << 9) | (sr1.to_u16().unwrap() << 6) | 0x5000 | imm | (1 << 5);
    vec![com]
}

pub fn encode_jmp(sr1: Register) -> Vec<u16> {
    let com = (sr1.to_u16().unwrap() << 6) | 0xc000;
    vec![com]
}

pub fn encode_jsr(offset: u16) -> Vec<u16> {
    let com = 0x4800 | offset;
    vec![com]
}

pub fn encode_jsrr(sr1: Register) -> Vec<u16> {
    let com = 0x4000 | (sr1.to_u16().unwrap() << 6);
    vec![com]
}

pub fn encode_ld(dr: Register, offset: u16) -> Vec<u16> {
    let com = (dr.to_u16().unwrap() << 9) | offset | 0x2000;
    vec![com]
}

pub fn encode_ldi(dr: Register, offset: u16) -> Vec<u16> {
    let com = (dr.to_u16().unwrap() << 9) | offset | 0xa000;
    vec![com]
}

pub fn encode_ldr(dr: Register, sr: Register, offset: u16) -> Vec<u16> {
    let com = (dr.to_u16().unwrap() << 9) | (sr.to_u16().unwrap() << 6) | offset | 0x6000;
    vec![com]
}

pub fn encode_lea(dr: Register, offset: u16) -> Vec<u16> {
    let com = (dr.to_u16().unwrap() << 9) | offset | 0xe000;
    vec![com]
}

pub fn encode_not(dr: Register, sr: Register) -> Vec<u16> {
    let com = (dr.to_u16().unwrap() << 9) | (sr.to_u16().unwrap() << 6) | 0x903f;
    vec![com]
}

pub fn encode_ret() -> Vec<u16> {
    vec![0b1100000111000000]
}

pub fn encode_rti() -> Vec<u16> {
    vec![0b1000000000000000]
}

pub fn encode_st(sr: Register, offset: u16) -> Vec<u16> {
    let com = (sr.to_u16().unwrap() << 9) | offset | 0x3000;
    vec![com]
}

pub fn encode_sti(sr: Register, offset: u16) -> Vec<u16> {
    let com = (sr.to_u16().unwrap() << 9) | offset | 0xb000;
    vec![com]
}

pub fn encode_str(sr: Register, baser: Register, offset: u16) -> Vec<u16> {
    let com = (sr.to_u16().unwrap() << 9) | (baser.to_u16().unwrap() << 6) | offset | 0x7000;
    vec![com]
}

pub fn encode_halt() -> Vec<u16> {
    vec![0xf025]
}

pub fn encode_in() -> Vec<u16> {
    vec![0xf023]
}

pub fn encode_out() -> Vec<u16> {
    vec![0xf021]
}

pub fn encode_getc() -> Vec<u16> {
    vec![0xf020]
}

pub fn encode_puts() -> Vec<u16> {
    vec![0xf022]
}

pub fn encode_putsp() -> Vec<u16> {
    vec![0xf024]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill() {
        assert_eq!(encode_fill(0x69), vec![0x69]);
        assert_eq!(encode_fill(0xDEAD), vec![0xDEAD]);
    }

    #[test]
    fn test_blkw() {
        assert_eq!(encode_blkw(5), vec![0; 5]);
        assert_eq!(encode_blkw(10), vec![0; 10]);
    }

    #[test]
    fn test_stringz() {
        assert_eq!(
            encode_stringz(String::from("Jibby World")),
            vec![74, 105, 98, 98, 121, 32, 87, 111, 114, 108, 100, 0]
        );
        assert_eq!(
            encode_stringz(String::from("Hello Debu")),
            vec![72, 101, 108, 108, 111, 32, 68, 101, 98, 117, 0]
        );
    }

    #[test]
    fn test_orig() {
        assert_eq!(encode_orig(0x69), vec![0x69]);
        assert_eq!(encode_orig(0xDEAD), vec![0xDEAD]);
    }

    #[test]
    fn test_add() {
        assert_eq!(
            encode_add_reg(Register::R1, Register::R5, Register::R2),
            vec![0b0001001101000010]
        );
        assert_eq!(
            encode_add_reg(Register::R0, Register::R4, Register::R0),
            vec![0b0001000100000000]
        );
        assert_eq!(
            encode_add_imm(Register::R7, Register::R2, 29),
            vec![0b0001111010111101]
        );
        assert_eq!(
            encode_add_imm(Register::R0, Register::R0, 0),
            vec![0b0001000000100000]
        );
    }

    #[test]
    fn test_and() {
        assert_eq!(
            encode_and_reg(Register::R1, Register::R5, Register::R2),
            vec![0b0101001101000010]
        );
        assert_eq!(
            encode_and_reg(Register::R0, Register::R4, Register::R0),
            vec![0b0101000100000000]
        );
        assert_eq!(
            encode_and_imm(Register::R7, Register::R2, 13),
            vec![0b0101111010101101]
        );
        assert_eq!(
            encode_and_imm(Register::R0, Register::R0, 0),
            vec![0b0101000000100000]
        );
    }

    #[test]
    fn test_jmp() {
        assert_eq!(encode_jmp(Register::R7), vec![0b1100000111000000]);
        assert_eq!(encode_jmp(Register::R0), vec![0b1100000000000000]);
        assert_eq!(encode_ret(), vec![0b1100000111000000]);
    }

    #[test]
    fn test_jsr() {
        assert_eq!(encode_jsr(713), vec![0b0100101011001001]);
        assert_eq!(encode_jsr(2047), vec![0b0100111111111111]);
    }

    #[test]
    fn test_jsrr() {
        assert_eq!(encode_jsrr(Register::R1), vec![0b0100000001000000]);
        assert_eq!(encode_jsrr(Register::R6), vec![0b0100000110000000]);
    }

    #[test]
    fn test_ld() {
        assert_eq!(encode_ld(Register::R2, 511), vec![0b0010010111111111]);
        assert_eq!(encode_ld(Register::R5, 0), vec![0b0010101000000000]);
    }

    #[test]
    fn test_ldi() {
        assert_eq!(encode_ldi(Register::R1, 420), vec![0b1010001110100100]);
        assert_eq!(encode_ldi(Register::R6, 69), vec![0b1010110001000101]);
    }

    #[test]
    fn test_ldr() {
        assert_eq!(
            encode_ldr(Register::R4, Register::R3, 53),
            vec![0b0110100011110101]
        );
        assert_eq!(
            encode_ldr(Register::R0, Register::R0, 14),
            vec![0b0110000000001110]
        );
    }

    #[test]
    fn test_lea() {
        assert_eq!(encode_lea(Register::R1, 313), vec![0b1110001100111001]);
        assert_eq!(encode_lea(Register::R6, 36), vec![0b1110110000100100]);
    }

    #[test]
    fn test_not() {
        assert_eq!(
            encode_not(Register::R1, Register::R5),
            vec![0b1001001101111111]
        );
        assert_eq!(
            encode_not(Register::R0, Register::R4),
            vec![0b1001000100111111]
        );
    }

    #[test]
    fn test_rti() {
        assert_eq!(encode_rti(), vec![0b1000000000000000]);
    }

    #[test]
    fn test_st() {
        assert_eq!(encode_st(Register::R5, 162), vec![0b0011101010100010]);
        assert_eq!(encode_st(Register::R7, 97), vec![0b0011111001100001]);
    }

    #[test]
    fn test_sti() {
        assert_eq!(encode_sti(Register::R3, 7), vec![0b1011011000000111]);
        assert_eq!(encode_sti(Register::R6, 31), vec![0b1011110000011111]);
    }

    #[test]
    fn test_str() {
        assert_eq!(
            encode_str(Register::R7, Register::R3, 33),
            vec![0b0111111011100001]
        );
        assert_eq!(
            encode_str(Register::R6, Register::R4, 22),
            vec![0b0111110100010110]
        );
    }

    #[test]
    fn test_halt() {
        assert_eq!(encode_halt(), vec![0xf025]);
    }

    #[test]
    fn test_in() {
        assert_eq!(encode_in(), vec![0xf023]);
    }

    #[test]
    fn test_out() {
        assert_eq!(encode_out(), vec![0xf021]);
    }

    #[test]
    fn test_getc() {
        assert_eq!(encode_getc(), vec![0xf020]);
    }

    #[test]
    fn test_puts() {
        assert_eq!(encode_puts(), vec![0xf022]);
    }

    #[test]
    fn test_putsp() {
        assert_eq!(encode_putsp(), vec![0xf024]);
    }
}
