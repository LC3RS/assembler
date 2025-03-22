use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use crate::{error::Result, utils::tokenize};

pub struct Assembler {
    file_path: PathBuf,
    lines: Option<Vec<String>>,
}

impl Assembler {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            lines: None,
        }
    }

    pub fn assemble(&mut self) -> Result<()> {
        println!("Starting assembly process...");
        self.read_file()?;
        self.first_pass()?;

        Ok(())
    }

    pub fn read_file(&mut self) -> Result<()> {
        let file = BufReader::new(File::open(&self.file_path)?);
        let lines: Vec<_> = file.lines().map(|l| l.unwrap()).collect();
        self.lines = Some(lines);

        Ok(())
    }

    pub fn first_pass(&self) -> Result<()> {
        let mut _lc: u16 = 0;

        if let Some(lines) = &self.lines {
            for line in lines {
                if let Some(tokens) = tokenize(line)? {
                    for token in &tokens {
                        print!("{:?} ", token);
                    }
                    println!();
                }
            }
        }

        Ok(())
    }
}
