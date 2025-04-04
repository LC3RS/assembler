use std::slice::Iter;

use num_derive::{FromPrimitive, ToPrimitive};

use crate::{
    error::{Error, ErrorKind, Result},
    utils::parse_constant,
};

#[repr(usize)]
#[derive(FromPrimitive, ToPrimitive, Clone, Copy, Debug, PartialEq)]
pub enum Register {
    R0 = 0x0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
}

#[repr(u16)]
#[derive(ToPrimitive, Clone, Copy, Debug, PartialEq)]
pub enum OpCode {
    Br = 0b0000000, // 0b0000 branch
    Brp,            // 0b0000 branch if n
    Brz,            // 0b0000 branch if z
    Brzp,           // 0b0000 branch if p
    Brn,            // 0b0000 branch if zp
    Brnp,           // 0b0000 branch if np
    Brnz,           // 0b0000 barnch if nz
    Brnzp,          // 0b0000 branch if nzp
    Add,            // 0b0001 add
    Ld,             // 0b0010 load
    St,             // 0b0011 store
    Jsr,            // 0b0100 jump register
    Jsrr,           // 0b0100 jump register
    And,            // 0b0101 bitwise and
    Ldr,            // 0b0110 load register
    Str,            // 0b0111 store register
    Rti,            // 0b1000 unused
    Not,            // 0b1001 bitwise not
    Ldi,            // 0b1010 load indirect
    Sti,            // 0b1011 store indirect
    Ret,            // 0b1100 return
    Jmp,            // 0b1100 jump
    Res,            // 0b1101 reserved
    Lea,            // 0b1110 load effective address
    Trap,           // 0b1111 execute trap

    /* Traps */
    GetC = 0x20, // 0x20
    Out,         // 0x21
    Puts,        // 0x22
    In,          // 0x23
    PutsP,       // 0x24
    Halt,        // 0x25
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Directive {
    Orig,
    End,
    Fill,
    Blkw,
    Stringz,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Label(String),
    Op(OpCode),
    Dir(Directive),
    Const(u16),
    Reg(Register),
    Str(String),
    Invalid,
}

pub trait Parseable {
    fn parse(s: &str) -> Result<Self>
    where
        Self: Sized;
}

impl Parseable for OpCode {
    fn parse(s: &str) -> Result<Self> {
        match s {
            "BR" => Ok(Self::Br),
            "BRN" => Ok(Self::Brn),
            "BRZ" => Ok(Self::Brz),
            "BRP" => Ok(Self::Brp),
            "BRZP" => Ok(Self::Brzp),
            "BRNP" => Ok(Self::Brnp),
            "BRNZ" => Ok(Self::Brnz),
            "BRNZP" => Ok(Self::Brnzp),
            "ADD" => Ok(Self::Add),
            "LD" => Ok(Self::Ld),
            "ST" => Ok(Self::St),
            "JSR" => Ok(Self::Jsr),
            "JSRR" => Ok(Self::Jsrr),
            "AND" => Ok(Self::And),
            "LDR" => Ok(Self::Ldr),
            "STR" => Ok(Self::Str),
            "RTI" => Ok(Self::Rti),
            "NOT" => Ok(Self::Not),
            "LDI" => Ok(Self::Ldi),
            "STI" => Ok(Self::Sti),
            "RET" => Ok(Self::Ret),
            "JMP" => Ok(Self::Jmp),
            "RES" => Ok(Self::Res),
            "LEA" => Ok(Self::Lea),
            "TRAP" => Ok(Self::Trap),
            "GETC" => Ok(Self::GetC),
            "OUT" => Ok(Self::Out),
            "PUTS" => Ok(Self::Puts),
            "IN" => Ok(Self::In),
            "PUTSP" => Ok(Self::PutsP),
            "HALT" => Ok(Self::Halt),

            _ => Err(Error::new(ErrorKind::ParseOpCodeError)),
        }
    }
}

impl Parseable for Register {
    fn parse(s: &str) -> Result<Self> {
        match s {
            "R0" => Ok(Self::R0),
            "R1" => Ok(Self::R1),
            "R2" => Ok(Self::R2),
            "R3" => Ok(Self::R3),
            "R4" => Ok(Self::R4),
            "R5" => Ok(Self::R5),
            "R6" => Ok(Self::R6),
            "R7" => Ok(Self::R7),

            _ => Err(Error::new(ErrorKind::ParseRegisterError)),
        }
    }
}

impl Parseable for Directive {
    fn parse(s: &str) -> Result<Self> {
        match s {
            ".ORIG" => Ok(Self::Orig),
            ".END" => Ok(Self::End),
            ".FILL" => Ok(Self::Fill),
            ".BLKW" => Ok(Self::Blkw),
            ".STRINGZ" => Ok(Self::Stringz),

            _ => Err(Error::new(ErrorKind::ParseDirectiveError)),
        }
    }
}

impl Parseable for Token {
    fn parse(mut s: &str) -> Result<Self> {
        s = s.trim();
        let upper = s.to_uppercase();

        let token = match upper.as_str() {
            // OpCode
            "BR" | "BRN" | "BRZ" | "BRP" | "BRZP" | "BRNP" | "BRNZ" | "BRNZP" | "ADD" | "LD"
            | "ST" | "JSR" | "JSRR" | "AND" | "LDR" | "STR" | "RTI" | "NOT" | "LDI" | "STI"
            | "RET" | "JMP" | "RES" | "LEA" | "TRAP" | "GETC" | "OUT" | "PUTS" | "IN" | "PUTSP"
            | "HALT" => Self::Op(OpCode::parse(&upper)?),

            // Register
            "R0" | "R1" | "R2" | "R3" | "R4" | "R5" | "R6" | "R7" => {
                Self::Reg(Register::parse(&upper)?)
            }

            // Directive
            ".ORIG" | ".END" | ".FILL" | ".BLKW" | ".STRINGZ" => {
                Self::Dir(Directive::parse(&upper)?)
            }

            _ => {
                if s.starts_with(['x', '#', 'b']) {
                    // Constant
                    Self::Const(parse_constant(s)?)
                } else if s.starts_with('"') && s.ends_with('"') {
                    // String
                    Self::Str(
                        s.to_owned()
                            .replace("\"", "")
                            .replace("\\t", "\t")
                            .replace("\\n", "\n")
                            .replace("\\e", "\x1b"),
                    )
                } else {
                    // Label
                    Self::Label(s.to_owned())
                }
            }
        };

        Ok(token)
    }
}

impl Token {
    /// Unwrap a Token::Const, return SyntaxError Otherwise
    pub fn take_const(&self) -> Result<u16> {
        if let Token::Const(c) = self {
            Ok(*c)
        } else {
            Err(Error::new(ErrorKind::SyntaxError))
        }
    }

    /// Unwrap a Token::Str, return SyntaxError Otherwise
    pub fn take_str(&self) -> Result<String> {
        if let Token::Str(s) = self {
            Ok(s.to_owned())
        } else {
            Err(Error::new(ErrorKind::SyntaxError))
        }
    }

    /// Unwrap a Token::Reg, return SyntaxError Otherwise
    pub fn take_reg(&self) -> Result<Register> {
        if let Token::Reg(r) = self {
            Ok(r.to_owned())
        } else {
            Err(Error::new(ErrorKind::SyntaxError))
        }
    }

    /// Unwrap a Token::Label, return SyntaxError Otherwise
    pub fn take_label(&self) -> Result<String> {
        if let Token::Label(l) = self {
            Ok(l.to_owned())
        } else {
            Err(Error::new(ErrorKind::SyntaxError))
        }
    }
}

/// Forces the next element of the iter to be present
/// Throws UnexpectedEof Otherwise
pub trait MustNext<T> {
    /// Returns reference to the next element of the iterator.
    /// May return UnexpectedEof if next element does not exist
    fn must_next(&mut self) -> Result<&T>;
}

/// Blanket implementation of MustNext over std::Slice::Iter
/// This makes it easier to expect arguements in encoder functions
impl<T> MustNext<T> for Iter<'_, T> {
    fn must_next(&mut self) -> Result<&T> {
        if let Some(item) = self.next() {
            Ok(item)
        } else {
            Err(Error::new(ErrorKind::UnexpectedEof))
        }
    }
}
