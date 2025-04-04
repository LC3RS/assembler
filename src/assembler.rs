use crate::encoder::{
    encode_add_imm, encode_add_reg, encode_and_imm, encode_and_reg, encode_br, encode_getc,
    encode_halt, encode_in, encode_jmp, encode_jsr, encode_jsrr, encode_ld, encode_ldi, encode_ldr,
    encode_lea, encode_not, encode_out, encode_puts, encode_putsp, encode_ret, encode_rti,
    encode_st, encode_sti, encode_str,
};
use crate::enums::OpCode;
use crate::utils::{resolve_dir, verify_offset};
use crate::{
    encoder::{encode_blkw, encode_fill, encode_orig, encode_stringz},
    enums::{Directive, MustNext, Token},
    error::{Error, ErrorKind, Result},
    utils::tokenize,
};
use byteorder::{BigEndian, WriteBytesExt};

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
    debug_mode: bool,
}

impl Assembler {
    pub fn new(file_path: PathBuf, outfile: String, debug_mode: bool) -> Self {
        Self {
            file_path,
            outfile,
            lines: None,
            tokens: Vec::new(),
            bin: Vec::new(),
            sym_table: HashMap::new(),
            debug_mode,
        }
    }

    pub fn assemble(&mut self) -> Result<()> {
        self.debug("Starting assembly process".to_string());
        self.read_file()?;
        self.first_pass()?;
        self.emit_sym_table()?;
        self.second_pass()?;
        self.emit_obj_file()?;

        Ok(())
    }

    pub fn debug(&self, s: String) {
        if self.debug_mode {
            println!("[Debug] {s}");
        }
    }

    fn read_file(&mut self) -> Result<()> {
        let file = BufReader::new(File::open(&self.file_path)?);
        let lines: Vec<_> = file.lines().map(|l| l.unwrap()).collect();
        self.lines = Some(lines);

        Ok(())
    }

    fn emit_sym_table(&self) -> Result<()> {
        let mut sym_path = resolve_dir();
        sym_path.push(format!("{}.sym", self.outfile));

        let mut file = BufWriter::new(File::create(sym_path)?);

        let mut labels: Vec<_> = self.sym_table.keys().map(|l| l.to_owned()).collect();
        labels.sort_by(|a, b| {
            self.sym_table
                .get(a)
                .unwrap()
                .cmp(self.sym_table.get(b).unwrap())
        });

        for label in labels {
            file.write_all(
                format!("{} {:x}\n", label, self.sym_table.get(&label).unwrap()).as_bytes(),
            )?;
        }
        file.flush()?;

        self.debug("Symbol Table".to_owned());
        self.debug(format!("{:#x?}", self.sym_table));

        Ok(())
    }

    fn emit_obj_file(&self) -> Result<()> {
        let mut bin_path = resolve_dir();
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
                    self.debug(format!("[{:x}] {:x?}", lc, tokens));

                    let idx = match &tokens[0] {
                        Token::Label(label) => {
                            self.sym_table.insert(label.clone(), lc);
                            1
                        }
                        _ => 0,
                    };

                    if idx < tokens.len() {
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
                    let arg = token_iter.must_next()?;
                    let v = match arg {
                        Token::Const(_) => arg.take_const()?,
                        Token::Label(l) => {
                            let x = self
                                .sym_table
                                .get(l)
                                .ok_or(Error::new(ErrorKind::MissingLabelError));
                            x?.to_owned()
                        }
                        _ => return Err(Error::new(ErrorKind::SyntaxError)),
                    };
                    lc += 1;
                    encode_fill(v)
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
                    let arg = token_iter.must_next()?;
                    lc += 1;
                    let offset = match arg {
                        Token::Const(_) => arg.take_const()?,
                        Token::Label(l) => {
                            let addr = self
                                .sym_table
                                .get(l)
                                .ok_or(Error::new(ErrorKind::MissingLabelError));
                            addr?.wrapping_sub(lc)
                        }
                        _ => return Err(Error::new(ErrorKind::SyntaxError)),
                    };
                    encode_br(token, verify_offset(offset, 9)?)
                }

                Token::Op(OpCode::Add) => {
                    let dr = token_iter.must_next()?.take_reg()?;
                    let sr1 = token_iter.must_next()?.take_reg()?;
                    let arg = token_iter.must_next()?;
                    let bin = match arg {
                        Token::Reg(_) => encode_add_reg(dr, sr1, arg.take_reg()?),
                        Token::Const(_) => {
                            encode_add_imm(dr, sr1, verify_offset(arg.take_const()?, 5)?)
                        }
                        _ => return Err(Error::new(ErrorKind::SyntaxError)),
                    };
                    lc += 1;
                    bin
                }

                Token::Op(OpCode::And) => {
                    let dr = token_iter.must_next()?.take_reg()?;
                    let sr1 = token_iter.must_next()?.take_reg()?;
                    let arg = token_iter.must_next()?;
                    let bin = match arg {
                        Token::Reg(_) => encode_and_reg(dr, sr1, arg.take_reg()?),
                        Token::Const(_) => {
                            encode_and_imm(dr, sr1, verify_offset(arg.take_const()?, 5)?)
                        }
                        _ => return Err(Error::new(ErrorKind::SyntaxError)),
                    };
                    lc += 1;
                    bin
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
                    lc += 1;
                    let offset = addr.wrapping_sub(lc);
                    encode_jsr(verify_offset(offset, 11)?)
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
                    lc += 1;
                    let offset = addr.wrapping_sub(lc);
                    encode_ld(dr, verify_offset(offset, 9)?)
                }

                Token::Op(OpCode::Ldi) => {
                    let dr = token_iter.must_next()?.take_reg()?;
                    let label = token_iter.must_next()?.take_label()?;
                    let addr = self
                        .sym_table
                        .get(&label)
                        .ok_or(Error::new(ErrorKind::MissingLabelError))?;
                    lc += 1;
                    let offset = addr.wrapping_sub(lc);
                    encode_ldi(dr, verify_offset(offset, 9)?)
                }

                Token::Op(OpCode::Ldr) => {
                    let dr = token_iter.must_next()?.take_reg()?;
                    let baser = token_iter.must_next()?.take_reg()?;
                    let offset = verify_offset(token_iter.must_next()?.take_const()?, 6)?;
                    lc += 1;
                    encode_ldr(dr, baser, offset)
                }

                Token::Op(OpCode::Lea) => {
                    let dr = token_iter.must_next()?.take_reg()?;
                    let label = token_iter.must_next()?.take_label()?;
                    let addr = self
                        .sym_table
                        .get(&label)
                        .ok_or(Error::new(ErrorKind::MissingLabelError))?;

                    lc += 1;
                    let offset = addr.wrapping_sub(lc);
                    encode_lea(dr, verify_offset(offset, 9)?)
                }

                Token::Op(OpCode::Not) => {
                    let dr = token_iter.must_next()?.take_reg()?;
                    let sr = token_iter.must_next()?.take_reg()?;
                    lc += 1;
                    encode_not(dr, sr)
                }

                Token::Op(OpCode::Ret) => {
                    lc += 1;
                    encode_ret()
                }

                Token::Op(OpCode::Rti) => {
                    lc += 1;
                    encode_rti()
                }

                Token::Op(OpCode::St) => {
                    let sr = token_iter.must_next()?.take_reg()?;
                    let label = token_iter.must_next()?.take_label()?;
                    let addr = self
                        .sym_table
                        .get(&label)
                        .ok_or(Error::new(ErrorKind::MissingLabelError))?;
                    lc += 1;
                    let offset = addr.wrapping_sub(lc);
                    encode_st(sr, verify_offset(offset, 9)?)
                }

                Token::Op(OpCode::Sti) => {
                    let sr = token_iter.must_next()?.take_reg()?;
                    let label = token_iter.must_next()?.take_label()?;
                    let addr = self
                        .sym_table
                        .get(&label)
                        .ok_or(Error::new(ErrorKind::MissingLabelError))?;
                    lc += 1;
                    let offset = addr.wrapping_sub(lc);
                    encode_sti(sr, verify_offset(offset, 9)?)
                }

                Token::Op(OpCode::Str) => {
                    let sr1 = token_iter.must_next()?.take_reg()?;
                    let sr2 = token_iter.must_next()?.take_reg()?;
                    lc += 1;
                    let offset = verify_offset(token_iter.must_next()?.take_const()?, 6)?;
                    encode_str(sr1, sr2, offset)
                }

                Token::Op(OpCode::Trap) => {
                    let trap_vec = token_iter.must_next()?.take_const()?;
                    lc += 1;
                    match trap_vec {
                        0x20 => encode_getc(),
                        0x21 => encode_out(),
                        0x22 => encode_puts(),
                        0x23 => encode_in(),
                        0x24 => encode_putsp(),
                        0x25 => encode_halt(),
                        _ => return Err(Error::new(ErrorKind::SyntaxError)),
                    }
                }

                Token::Op(OpCode::GetC) => {
                    lc += 1;
                    encode_getc()
                }

                Token::Op(OpCode::Puts) => {
                    lc += 1;
                    encode_puts()
                }

                Token::Op(OpCode::PutsP) => {
                    lc += 1;
                    encode_putsp()
                }

                Token::Op(OpCode::In) => {
                    lc += 1;
                    encode_in()
                }

                Token::Op(OpCode::Out) => {
                    lc += 1;
                    encode_out()
                }

                Token::Op(OpCode::Halt) => {
                    lc += 1;
                    encode_halt()
                }

                Token::Label(_) | Token::Op(OpCode::Res) => continue,

                // Orphan constants, registers or strings should be syntax error
                Token::Str(_) | Token::Reg(_) | Token::Const(_) => {
                    return Err(Error::new(ErrorKind::SyntaxError))
                }

                Token::Invalid => return Err(Error::new(ErrorKind::InvalidTokenError)),
            };

            self.bin.append(&mut bin);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;
    use byteorder::ReadBytesExt;

    #[test]
    fn test_assembler_basic() {
        let mut test_ass =
            Assembler::new(PathBuf::from("asm/test.ggnm"), String::from("test"), true);

        let res = test_ass.read_file();
        assert!(res.is_ok());

        let res = test_ass.first_pass();
        assert!(res.is_ok());

        assert_eq!(test_ass.sym_table.get("HELLO_WORLD"), Some(&0x3003));

        let res = test_ass.emit_sym_table();
        assert!(res.is_ok());

        let mut sym_path = resolve_dir();
        sym_path.push("test.sym");
        let f = File::open(sym_path);
        assert!(f.is_ok());
        let mut f = f.unwrap();
        let mut sym_file_content = String::new();
        f.read_to_string(&mut sym_file_content).unwrap();

        let f = File::open("roms/test.sym");
        assert!(f.is_ok());
        let mut f = f.unwrap();
        let mut expected_sym_content = String::new();
        f.read_to_string(&mut expected_sym_content).unwrap();

        assert_eq!(&sym_file_content[..], &expected_sym_content[..]);

        let res = test_ass.second_pass();
        assert!(res.is_ok());

        let mut file = BufReader::new(File::open("roms/test.obj").unwrap());
        let mut expected: Vec<u16> = vec![];
        while let Ok(word) = file.read_u16::<BigEndian>() {
            expected.push(word);
        }

        assert_eq!(&test_ass.bin[..], &expected[..]);
    }

    #[test]
    fn test_assembler_instructions() {
        let mut test_ass = Assembler::new(
            PathBuf::from("asm/instructions.ggnm"),
            String::from("instructions"),
            false,
        );

        let res = test_ass.read_file();
        assert!(res.is_ok());

        let res = test_ass.first_pass();
        assert!(res.is_ok());

        assert_eq!(test_ass.sym_table.get("LABEL0"), Some(&0x302a));
        assert_eq!(test_ass.sym_table.get("LABEL3"), Some(&0x302d));
        assert_eq!(test_ass.sym_table.get("HELLO_WORLD"), Some(&0x3002));
        assert_eq!(test_ass.sym_table.get("LABEL4"), Some(&0x302e));
        assert_eq!(test_ass.sym_table.get("LABEL6"), Some(&0x3030));
        assert_eq!(test_ass.sym_table.get("LABEL2"), Some(&0x302c));
        assert_eq!(test_ass.sym_table.get("LABEL5"), Some(&0x302f));
        assert_eq!(test_ass.sym_table.get("LABEL7"), Some(&0x3031));
        assert_eq!(test_ass.sym_table.get("LABEL1"), Some(&0x302b));

        let res = test_ass.emit_sym_table();
        assert!(res.is_ok());

        let mut sym_path = resolve_dir();
        sym_path.push("instructions.sym");
        let f = File::open(sym_path);
        assert!(f.is_ok());
        let mut f = f.unwrap();
        let mut sym_file_content = String::new();
        f.read_to_string(&mut sym_file_content).unwrap();

        let f = File::open("roms/instructions.sym");
        assert!(f.is_ok());
        let mut f = f.unwrap();
        let mut expected_sym_content = String::new();
        f.read_to_string(&mut expected_sym_content).unwrap();

        assert_eq!(&sym_file_content[..], &expected_sym_content[..]);

        let res = test_ass.second_pass();
        assert!(res.is_ok());

        let mut file = BufReader::new(File::open("roms/instructions.obj").unwrap());
        let mut expected: Vec<u16> = vec![];
        while let Ok(word) = file.read_u16::<BigEndian>() {
            expected.push(word);
        }

        assert_eq!(&test_ass.bin[..], &expected[..]);
    }

    #[test]
    fn test_assembler_2048() {
        let mut test_ass =
            Assembler::new(PathBuf::from("asm/2048.ggnm"), String::from("2048"), false);

        let res = test_ass.assemble();
        assert!(res.is_ok());

        let mut sym_path = resolve_dir();
        sym_path.push("2048.sym");
        let f = File::open(sym_path);
        assert!(f.is_ok());
        let mut f = f.unwrap();
        let mut sym_file_content = String::new();
        f.read_to_string(&mut sym_file_content).unwrap();

        let f = File::open("roms/2048.sym");
        assert!(f.is_ok());
        let mut f = f.unwrap();
        let mut expected_sym_content = String::new();
        f.read_to_string(&mut expected_sym_content).unwrap();

        assert_eq!(&sym_file_content[..], &expected_sym_content[..]);

        let mut file = BufReader::new(File::open("roms/2048.obj").unwrap());
        let mut expected: Vec<u16> = vec![];
        while let Ok(word) = file.read_u16::<BigEndian>() {
            expected.push(word);
        }

        assert_eq!(&test_ass.bin[..], &expected[..]);
    }

    #[test]
    fn fault_testing() {
        let syntax_tests = 9;
        for i in 1..=syntax_tests {
            let mut test_ass = Assembler::new(
                PathBuf::from(format!("asm/fault_tests/syntax_fault-{i}.ggnm")),
                format!("syntax_fault-{i}"),
                false,
            );
            let res = test_ass.assemble().map_err(|e| e.kind);
            assert_eq!(res, Err(ErrorKind::SyntaxError));
        }
    }
}
