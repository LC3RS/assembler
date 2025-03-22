use crate::{
    enums::{Parseable, Token},
    error::Result,
};

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

pub fn parse_constant(s: &str) -> Result<i32> {
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

    Ok(res)
}
