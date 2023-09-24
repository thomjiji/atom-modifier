use std::fs::File;
use std::io;
use std::io::{BufReader, Error, Read, Seek, SeekFrom, Write};

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
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0; pattern.len()];
        let mut offset = 0;

        while reader.read(&mut buffer)? != 0 {
            if buffer.as_slice() == pattern {
                let mut atom = Self::new();

                atom.offset += offset; // set offset

                reader.seek(SeekFrom::Current(-8))?; // The size of colr atom is 8 bytes before the colr's type.
                let mut size_buf = [0; 4];
                reader.read_exact(&mut size_buf)?;
                atom.size += u32::from_be_bytes(size_buf); // set size

                reader.seek(SeekFrom::Start(offset))?;
                return Ok(atom);
            }
            offset += 1;
            reader.seek(SeekFrom::Start(offset))?;
        }

        // println!("Pattern {:?} was not found in the file.", pattern);
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
    let mut stream =
        File::open("/Users/thom/code/rust/atom_modifier/test_footages/1-2-1_modified.mov")
            .expect("Failed to open the file");

    match ColrAtom::search(&mut stream, &COLR_ATOM) {
        Ok(atom) => {
            println!("Found atom: {:?}", atom);
        }
        Err(e) => {
            println!("An error occurred: {}", e);
        }
    };
}