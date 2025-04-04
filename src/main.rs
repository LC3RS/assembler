use clap::Parser;
use cli::Cli;

pub mod assembler;
pub mod cli;
pub mod encoder;
pub mod enums;
pub mod error;
pub mod utils;

fn main() {
    let args = Cli::parse();

    let mut asm = assembler::Assembler::new(args.file, args.outfile, args.debug);
    if let Err(e) = asm.assemble() {
        println!("{e}");
    }
}
