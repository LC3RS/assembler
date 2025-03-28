use crate::{
    enums::{Parseable, Token},
    error::{Error, ErrorKind, Result},
};

/// Parase a string into a vector of tokens
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
        u16::from_str_radix(s, 16)
    } else if let Some(s) = s.strip_prefix("b") {
        // Binary Constant
        u16::from_str_radix(s, 2)
    } else if let Some(s) = s.strip_prefix("#") {
        // Decimal Constant
        s.parse()
    } else {
        // Fallback error
        s.parse()
    }?;

    Ok(res)
}

/// Extend low bit numbers to 16bit (u16)
pub fn sign_extend(mut x: u16, bit_count: u16) -> u16 {
    if !(x >> bit_count) != 0 && x >> bit_count != 0 {
        return 0;
    }

    // Early return if bit_count is 0
    if bit_count == 0 {
        return x;
    }

    if ((x >> (bit_count - 1)) & 1) != 0 {
        x |= (0xFFFF) << bit_count;
    }
    x
}

/// Validate offset based on bit count
pub fn verify_offset(mut offset: u16, bit_count: u16) -> Result<()> {
    offset >>= bit_count;
    let cmp = (!0u16) >> bit_count;
    dbg!(cmp);
    if offset != cmp && offset != 0 {
        return Err(Error::new(ErrorKind::SyntaxError));
    }
    Ok(())
}
