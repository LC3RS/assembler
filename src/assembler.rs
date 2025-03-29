use crate::encoder::{
    encode_add, encode_and, encode_br, encode_jmp, encode_jsr, encode_jsrr, encode_ld,
};
use crate::enums::OpCode;
use crate::utils::{sign_extend, verify_offset};
use crate::{
    encoder::{encode_blkw, encode_fill, encode_orig, encode_stringz},
    enums::{Directive, MustNext, Token},
    error::{Error, ErrorKind, Result},
    utils::tokenize,
};
use byteorder::{BigEndian, WriteBytesExt};
use num_traits::ToPrimitive;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

pub struct Assembler {
    file_path: PathBuf,
    lines: Option<Vec<String>>,
    sym_table: HashMap<String, u16>,
    tokens: Vec<Token>,
    bin: Vec<u16>,
    outfile: String,
}

impl Assembler {
    pub fn new(file_path: PathBuf, outfile: String) -> Self {
        Self {
            file_path,
            outfile,
            lines: None,
            tokens: Vec::new(),
            bin: Vec::new(),
            sym_table: HashMap::new(),
        }
    }

    pub fn assemble(&mut self) -> Result<()> {
        println!("Starting assembly process...");
        self.read_file()?;
        self.first_pass()?;
        self.emit_sym_table()?;
        self.second_pass()?;
        self.emit_obj_file()?;

        Ok(())
    }

    fn read_file(&mut self) -> Result<()> {
        let file = BufReader::new(File::open(&self.file_path)?);
        let lines: Vec<_> = file.lines().map(|l| l.unwrap()).collect();
        self.lines = Some(lines);

        Ok(())
    }

    fn emit_sym_table(&self) -> Result<()> {
        let mut sym_path = PathBuf::new();
        if let Some(dirname) = self.file_path.parent() {
            sym_path.push(dirname);
        }
        sym_path.push(format!("{}.sym", self.outfile));

        let mut file = BufWriter::new(File::create(sym_path)?);
        for (label, lc) in &self.sym_table {
            file.write_all(format!("{} {:x}\n", label, lc).as_bytes())?;
        }
        file.flush()?;

        println!("Symbol Table");
        println!("{:#x?}", self.sym_table);

        Ok(())
    }

    fn emit_obj_file(&self) -> Result<()> {
        let mut bin_path = PathBuf::new();
        if let Some(dirname) = self.file_path.parent() {
            bin_path.push(dirname);
        }
        bin_path.push(format!("{}.obj", self.outfile));

        let mut file = BufWriter::new(File::create(bin_path)?);
        for &word in &self.bin {
            file.write_u16::<BigEndian>(word)?;
        }
        file.flush()?;

        Ok(())
    }

    fn first_pass(&mut self) -> Result<()> {
        let mut lc: u16 = 0;

        if let Some(lines) = &self.lines {
            for line in lines {
                if let Some(mut tokens) = tokenize(line)? {
                    // TODO: debug mode
                    print!("[{:x}] ", lc);
                    for token in &tokens {
                        print!("{:?} ", token);
                    }
                    println!();

                    let idx = match &tokens[0] {
                        Token::Label(label) => {
                            self.sym_table.insert(label.clone(), lc);
                            1
                        }
                        _ => 0,
                    };

                    // Empty labels should be a syntax error
                    // So throwing syntax error if the only token on a line is a label
                    if idx >= tokens.len() {
                        return Err(Error::new(ErrorKind::SyntaxError));
                    }

                    match &tokens[idx] {
                        Token::Dir(Directive::Orig) => {
                            if let Token::Const(c) = tokens[idx + 1] {
                                lc = c;
                            }
                        }
                        Token::Dir(Directive::Blkw) => {
                            if let Token::Const(c) = tokens[idx + 1] {
                                lc += c;
                            }
                        }
                        Token::Dir(Directive::Stringz) => {
                            if let Token::Str(s) = &tokens[idx + 1] {
                                lc += s.len() as u16 + 1; // +1 for the extra null-byte at the end
                            }
                        }

                        Token::Dir(Directive::End) => break,

                        _ => {
                            lc += 1;
                        }
                    }

                    self.tokens.append(&mut tokens);
                }
            }
        }

        Ok(())
    }

    fn second_pass(&mut self) -> Result<()> {
        let mut token_iter = self.tokens.iter();
        let mut lc;

        // Tokens should begin with Dir(Orig) and Const(c)
        // Otherwise syntax error
        let mut bin = match token_iter.next() {
            Some(Token::Dir(Directive::Orig)) => {
                let origin = token_iter.must_next()?.take_const()?;
                lc = origin;
                encode_orig(origin)
            }
            _ => return Err(Error::new(ErrorKind::SyntaxError)),
        };
        self.bin.append(&mut bin);

        while let Some(token) = token_iter.next() {
            let mut bin = match token {
                /* Directive Encoders */
                Token::Dir(Directive::Fill) => {
                    lc += 1u16;
                    encode_fill()
                }

                Token::Dir(Directive::Blkw) => {
                    let c = token_iter.must_next()?.take_const()?;
                    lc += c;
                    encode_blkw(c)
                }

                Token::Dir(Directive::Stringz) => {
                    let s = token_iter.must_next()?.take_str()?;
                    lc += s.len() as u16 + 1;
                    encode_stringz(s)
                }

                Token::Dir(Directive::Orig) => {
                    return Err(Error::new(ErrorKind::SyntaxError));
                }

                Token::Dir(Directive::End) => break,

                Token::Op(
                    OpCode::Br
                    | OpCode::Brn
                    | OpCode::Brnp
                    | OpCode::Brp
                    | OpCode::Brz
                    | OpCode::Brnz
                    | OpCode::Brzp
                    | OpCode::Brnzp,
                ) => {
                    let label = token_iter.must_next()?.take_label()?;
                    let addr = self
                        .sym_table
                        .get(&label)
                        .ok_or(Error::new(ErrorKind::MissingLabelError))?;
                    let offset = addr - lc;
                    lc += 1;
                    verify_offset(offset, 9)?;
                    encode_br(token, sign_extend(offset, 9))
                }

                Token::Op(OpCode::Add) => {
                    let dr = token_iter.must_next()?.take_reg()?;
                    let sr1 = token_iter.must_next()?.take_reg()?;
                    let sr2 = match token_iter.must_next()? {
                        Token::Reg(r) => r.to_u16().unwrap(),
                        Token::Const(c) => sign_extend(*c, 5) | 0b100000,
                        _ => return Err(Error::new(ErrorKind::SyntaxError)),
                    };
                    lc += 1;
                    encode_add(dr, sr1, sr2)
                }

                Token::Op(OpCode::And) => {
                    let dr = token_iter.must_next()?.take_reg()?;
                    let sr1 = token_iter.must_next()?.take_reg()?;
                    let sr2 = match token_iter.must_next()? {
                        Token::Reg(r) => r.to_u16().unwrap(),
                        Token::Const(c) => sign_extend(*c, 5) | 0b100000,
                        _ => return Err(Error::new(ErrorKind::SyntaxError)),
                    };
                    lc += 1;
                    encode_and(dr, sr1, sr2)
                }

                Token::Op(OpCode::Jmp) => {
                    let sr1 = token_iter.must_next()?.take_reg()?;
                    lc += 1;
                    encode_jmp(sr1)
                }

                Token::Op(OpCode::Jsr) => {
                    let label = token_iter.must_next()?.take_label()?;
                    let addr = self
                        .sym_table
                        .get(&label)
                        .ok_or(Error::new(ErrorKind::MissingLabelError))?;
                    let offset = addr - lc;
                    lc += 1;
                    verify_offset(offset, 11)?;
                    encode_jsr(sign_extend(offset, 11))
                }

                Token::Op(OpCode::Jsrr) => {
                    let sr1 = token_iter.must_next()?.take_reg()?;
                    lc += 1;
                    encode_jsrr(sr1)
                }

                Token::Op(OpCode::Ld) => {
                    let dr = token_iter.must_next()?.take_reg()?;
                    let label = token_iter.must_next()?.take_label()?;
                    let addr = self
                        .sym_table
                        .get(&label)
                        .ok_or(Error::new(ErrorKind::MissingLabelError))?;
                    let offset = addr - lc;
                    lc += 1;
                    verify_offset(offset, 9)?;
                    encode_ld(dr, sign_extend(offset, 9))
                }

                Token::Label(_) => continue,

                // Orphan constants, registers or strings should be syntax error
                Token::Str(_) | Token::Reg(_) | Token::Const(_) => {
                    return Err(Error::new(ErrorKind::SyntaxError))
                }

                _ => {
                    println!("Not yet implemented!");
                    Vec::new()
                }
            };

            self.bin.append(&mut bin);
        }

        Ok(())
    }
}
