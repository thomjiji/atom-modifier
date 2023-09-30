use std::fs::File;
use std::io;
use std::io::{Error, Read, Seek, SeekFrom, Write};
use std::time::Instant;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_file_path: String,
    #[arg(short, long)]
    colr: String,
    #[arg(short, long, default_value_t = String::from("0"))]
    gama: String,
}

static COLR_ATOM: [u8; 4] = [0x63, 0x6f, 0x6c, 0x72]; // "colr"
static GAMA_ATOM: [u8; 4] = [0x67, 0x61, 0x6d, 0x61]; // "gama"

#[derive(Debug)]
enum ColorParameterType {
    Nclc, // for video
    Prof, // for print
    Unknown,
}

#[derive(Debug)]
struct ColrAtom {
    size: u32,
    color_parameter_type: ColorParameterType,
    offset: u64,
    primaries: u16,
    transfer_function: u16,
    matrix: u16,
}

impl ColrAtom {
    fn new() -> ColrAtom {
        Self {
            size: 0,
            color_parameter_type: ColorParameterType::Nclc,
            offset: 0,
            primaries: 0,
            transfer_function: 0,
            matrix: 0,
        }
    }

    fn search(file: &mut File, pattern: &[u8]) -> Result<Self, Error> {
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let pattern_position = buffer
            .windows(pattern.len())
            .position(|window| window == pattern);

        if let Some(offset) = pattern_position {
            let mut atom = Self::new();

            // Set the offset to the position on the pattern match 4 bytes forward.
            atom.offset = (offset - 4) as u64;

            // From there until the next 4 bytes are size.
            atom.size = u32::from_be_bytes(buffer[offset - 4..offset].try_into().unwrap());

            atom.primaries =
                u16::from_be_bytes(buffer[offset + 8..offset + 10].try_into().unwrap());

            atom.transfer_function =
                u16::from_be_bytes(buffer[offset + 10..offset + 12].try_into().unwrap());

            atom.matrix = u16::from_be_bytes(buffer[offset + 12..offset + 14].try_into().unwrap());

            return Ok(atom);
        }

        Err(Error::new(
            io::ErrorKind::NotFound,
            "Atom pattern was not found in the file.",
        ))
    }
}

struct GamaAtom {
    size: u8,
    data: u32,
}

fn write_bytes_at(f: &mut File, position: u64, bytes: &[u8]) -> io::Result<()> {
    f.seek(SeekFrom::Start(position))?;
    f.write_all(bytes)
}

fn main() {
    let args = Args::parse();

    let mut stream = match File::open(args.input_file_path) {
        Ok(file) => {
            println!("File opened...");
            file
        }
        Err(e) => panic!("An error occurred when open file: {}", e),
    };

    let start = Instant::now();
    match ColrAtom::search(&mut stream, &COLR_ATOM) {
        Ok(atom) => {
            println!("Found atom: \n\t{:?}", atom);
        }
        Err(e) => {
            println!("An error occurred: {}", e);
        }
    };
    let duration = start.elapsed();
    println!(
        "Time elapsed in this search implementation is: {:?}",
        duration
    );
}
