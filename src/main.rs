use std::fs;


use bytes::Bytes;
use clap::{Subcommand, Parser};

mod huffman;
use huffman::Huffman;


enum Operation {
    Compress,
    Decompress
}



/// Shitty Algorithm for Compressing Admittedly Large Objects (Sacalo)
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}


#[derive(Subcommand)]
enum Commands {
    /// Compresses a file
    Compress {file: String, output: Option<String>},

    /// Decompresses a file
    Decompress {file: String, output: Option<String>}
}







fn main() {
    let cli = Cli::parse();

    let (op, input, output) = match cli.command {
        Commands::Compress { file, output } => (Operation::Compress, file, output),
        Commands::Decompress { file, output } => (Operation::Decompress, file, output),
    };

    // load data from file
    let data = match fs::read(&input) {
        Ok(d) => Bytes::from(d),
        Err(_) => {
            println!("\x1b[31m[ERROR] Unable to load file {}\x1b[0m", input);
            std::process::exit(1)
        },
    };

    // compress or decompress depending on the operation
    let result = match &op {
        Operation::Compress => Huffman::compress(&data),
        Operation::Decompress => Huffman::decompress(&data),
    };

    let result = match (result, &op) {
        (None, Operation::Compress) => {
            println!("\x1b[31m[ERROR] Error while compressing {}\x1b[0m", input);
            std::process::exit(1);
        },
        (None, Operation::Decompress) => {
            println!("\x1b[31m[ERROR] Unable to decompress {}. Was the file corrupted?\x1b[0m", input);
            std::process::exit(1);
        },
        (Some(r), ..) => r
    };


    // write the resulting file to an output
    let output_name = match (output, op) {
        (Some(n), _) => n,
        (None, Operation::Compress) => format!("{input}.scl"),
        (None, Operation::Decompress) => {
            match input.strip_suffix(".scl") {
                Some(n) => n.to_string(),
                None => format!("{input}_decompressed"),
            }
        }
    };



    match fs::write(&output_name, result) {
        Ok(_) => (),
        Err(_) => {
            println!("\x1b[31m[ERROR] Unable to write to {}\x1b[0m", output_name);
            std::process::exit(1);
        },
    }
}