use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

use crate::{
    enums::{Directive, Token},
    error::Result,
    utils::tokenize,
};

pub struct Assembler {
    file_path: PathBuf,
    lines: Option<Vec<String>>,
    sym_table: HashMap<String, u16>,
    outfile: String,
}

impl Assembler {
    pub fn new(file_path: PathBuf, outfile: String) -> Self {
        Self {
            file_path,
            outfile,
            lines: None,
            sym_table: HashMap::new(),
        }
    }

    pub fn assemble(&mut self) -> Result<()> {
        println!("Starting assembly process...");
        self.read_file()?;
        self.first_pass()?;

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
                if let Some(tokens) = tokenize(line)? {
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

                    match &tokens[idx] {
                        Token::Dir(Directive::Orig) => {
                            if let Token::Const(c) = tokens[idx + 1] {
                                lc = c as u16;
                            }
                        }
                        Token::Dir(Directive::Blkw) => {
                            if let Token::Const(c) = tokens[idx + 1] {
                                lc += c as u16;
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
            }
        }

        self.emit_sym_table()?;

        Ok(())
    }
}
