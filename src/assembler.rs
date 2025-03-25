use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

use crate::{
    encoder::{encode_blkw, encode_fill, encode_orig, encode_stringz},
    enums::{Directive, MustNext, Token},
    error::{Error, ErrorKind, Result},
    utils::tokenize,
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
        self.second_pass()?;

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

        self.emit_sym_table()?;

        Ok(())
    }

    fn second_pass(&mut self) -> Result<()> {
        let mut token_iter = self.tokens.iter();

        // Tokens should begin with Dir(Orig) and Const(c)
        // Otherwise syntax error
        let mut bin = match token_iter.next() {
            Some(Token::Dir(Directive::Orig)) => {
                let origin = token_iter.must_next()?.take_const()?;
                encode_orig(origin)
            }
            _ => return Err(Error::new(ErrorKind::SyntaxError)),
        };
        self.bin.append(&mut bin);

        while let Some(token) = token_iter.next() {
            let mut bin = match token {
                /* Directive Encoders */
                Token::Dir(Directive::Fill) => encode_fill(),

                Token::Dir(Directive::Blkw) => {
                    let c = token_iter.must_next()?.take_const()?;
                    encode_blkw(c)
                }

                Token::Dir(Directive::Stringz) => {
                    let s = token_iter.must_next()?.take_str()?;
                    encode_stringz(s)
                }

                Token::Dir(Directive::Orig) => {
                    return Err(Error::new(ErrorKind::SyntaxError));
                }

                Token::Dir(Directive::End) => break,

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
