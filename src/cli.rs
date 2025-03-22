use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to source assembly file
    ///
    /// Input file extensions should generally be .asm or .ggnm,
    /// but it's not strictly checked
    #[arg(short, long)]
    pub file: PathBuf,

    /// Output file name (without extension)
    ///
    /// Assembler emits <OUTFILE>.obj and <OUTFILE>.sym
    #[arg(short, long, default_value_t = String::from("out"))]
    pub outfile: String,

    /// Turn on debug-mode
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,
}
