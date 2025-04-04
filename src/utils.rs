use std::{env, path::PathBuf};

use crate::{
    enums::{Parseable, Token},
    error::{Error, ErrorKind, Result},
};

/// Parse a string into a vector of tokens
pub fn tokenize(s: &str) -> Result<Option<Vec<Token>>> {
    let s = s.trim();

    // Empty lines and comments are treated as blanks
    if s.is_empty() || s.starts_with(";") {
        return Ok(None);
    }

    // Ignore comments at the end (if present)
    let s = s.split(';').next().unwrap().trim();

    let mut tokens: Vec<Token> = Vec::new();
    let mut split = s.splitn(2, char::is_whitespace);

    if let Some(word) = split.next() {
        let token = Token::parse(word)?;
        tokens.push(token.clone());

        if let Some(rest) = split.next() {
            match token {
                Token::Op(_) => {
                    for arg in rest.split(',') {
                        tokens.push(Token::parse(arg)?)
                    }
                }
                Token::Dir(_) => tokens.push(Token::parse(rest)?),
                Token::Label(_) => {
                    if let Some(mut rest_tokens) = tokenize(rest)? {
                        tokens.append(&mut rest_tokens);
                    }
                }
                _ => {}
            }
        }

        Ok(Some(tokens))
    } else {
        Ok(None)
    }
}

/// Parse constants based on prefix
pub fn parse_constant(s: &str) -> Result<u16> {
    let res = if let Some(s) = s.strip_prefix("x") {
        // Hex Constant
        i32::from_str_radix(s, 16)
    } else if let Some(s) = s.strip_prefix("b") {
        // Binary Constant
        i32::from_str_radix(s, 2)
    } else if let Some(s) = s.strip_prefix("#") {
        // Decimal Constant
        s.parse()
    } else {
        // Fallback error
        s.parse()
    }?;

    Ok(res as u16)
}

/// Validate offset based on bit count
pub fn verify_offset(mut offset: u16, bit_count: u16) -> Result<u16> {
    let result = offset;
    offset >>= bit_count;
    let cmp = 0xffff >> bit_count;
    if offset != cmp && offset != 0 {
        return Err(Error::new(ErrorKind::ValueError));
    }

    Ok(result & (0xffff >> (16 - bit_count)))
}

///  Resolve directory dirname
pub fn resolve_dir() -> PathBuf {
    if cfg!(test) {
        let tmpdir = env::var("ASSEMBLER_TMP_DIR").unwrap_or(String::from("./tmp"));
        PathBuf::from(tmpdir)
    } else if let Ok(dirname) = env::current_dir() {
        dirname
    } else {
        PathBuf::from("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{Directive, OpCode, Register};

    #[test]
    fn test_tokenize() {
        let check1 = vec![
            Token::Label(String::from("HELLO_WORLD")),
            Token::Dir(Directive::Stringz),
            Token::Str(String::from("Hello, World")),
        ];
        assert_eq!(
            tokenize("HELLO_WORLD .stringz \"Hello, World\""),
            Ok(Some(check1))
        );
        let check2 = vec![
            Token::Op(OpCode::Add),
            Token::Reg(Register::R0),
            Token::Reg(Register::R2),
            Token::Const(14),
        ];
        assert_eq!(tokenize("ADD R0,R2,#14 ;COMMENT"), Ok(Some(check2)));
        assert_eq!(tokenize(";COMMENT2"), Ok(None));
    }

    #[test]
    fn test_parse_constant() {
        assert_eq!(parse_constant("#412"), Ok(412u16));
        assert_eq!(parse_constant("xa4"), Ok(0xa4u16));
        assert_eq!(parse_constant("b110101"), Ok(0b110101u16));
        assert_eq!(
            parse_constant("#f").map_err(|e| e.kind),
            Err(ErrorKind::ParseConstantError)
        );
    }

    #[test]
    fn test_verify_offset() {
        assert_eq!(verify_offset(0xffd4, 8), Ok(0x00d4));
        assert_eq!(
            verify_offset(0x0fff, 8).map_err(|e| e.kind),
            Err(ErrorKind::ValueError)
        );
    }
}
