use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};

use clap::Parser;

use crate::ColorParameterType::Nclc;

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

static COLR_ATOM: [u8; 4] = [0x63, 0x6F, 0x6C, 0x72]; // "colr"
static NCLC: [u8; 4] = [0x6e, 0x63, 0x6c, 0x63]; // "colr"

struct Video {
    file: File,
    size: u64,
    offset: u64,
    cursor: u64,
}

enum ColorParameterType {
    Nclc, // for video
    Prof, // for print
    Unknown,
}

struct ColrAtom {
    size: u8,
    parameter_type: ColorParameterType,
    offset: u64,
    primaries: u16,
    transfer_function: u16,
    matrix: u16,
}

impl ColrAtom {
    fn new() -> ColrAtom {
        Self {
            size: 0,
            parameter_type: Nclc,
            offset: 0,
            primaries: 0,
            transfer_function: 0,
            matrix: 0,
        }
    }
    fn search(file: &mut File, pattern: &[u8]) -> std::io::Result<Option<u64>> {
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0; pattern.len()];
        let mut offset = 0;

        while reader.read(&mut buffer)? != 0 {
            if buffer.as_slice() == pattern {
                return Ok(Some(offset));
            }
            offset += 1;
            reader.seek(SeekFrom::Start(offset))?;
        }

        println!("Pattern was not found in the file.");
        Ok(None)
    }
}

struct GamaAtom {
    size: u8,
    data: u32,
}

/// write_bytes_at(&mut file, position + 6, &[0x33, 0x33, 0x00, 0x00])?;  // change gama to 2.2
/// write_bytes_at(&mut file, position + 6, &[0x66, 0x66, 0x00, 0x00])?;  // change gama to 2.4
/// write_bytes_at(&mut file, position, &[0x00, 0x00, 0x00, 0x00])?;      // remove gama atom

fn write_bytes_at(f: &mut std::fs::File, position: u64, bytes: &[u8]) -> std::io::Result<()> {
    f.seek(SeekFrom::Start(position))?;
    f.write_all(bytes)
}

fn main() {
    let mut stream =
        File::open("/Users/thom/code/rust/atom_modifier/test_footages/1-2-1_modified.mov")
            .expect("Failed to open the file");
    let x = ColrAtom::search(&mut stream, &COLR_ATOM);
    println!("{:?}", x);

    // let mut stream = config.read_file("/Users/thom/Desktop/1-1-1.mov").unwrap();
    // let mut buffer = Vec::new();
    // let x = stream.read_to_end(&mut buffer).unwrap();
    //
    // for byte in buffer.iter().take(100) {
    //     println!("{} ", byte);
    // }
}