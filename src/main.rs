use std::fs::File;
use std::io;
use std::io::{Error, Read, Seek, SeekFrom, Write};

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
        let mut buffer = vec![0; 1024 * 1024];
        let mut file_content: Vec<u8> = Vec::new();
        let mut offset = 0;

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            file_content.extend_from_slice(&buffer[..bytes_read]);

            if let Some(index) = file_content
                .windows(pattern.len())
                .position(|window| window == pattern)
            {
                let mut atom = Self::new();

                let atom_offset = offset + (index as u64);

                atom.offset = atom_offset - 4;

                atom.size = u32::from_be_bytes(
                    file_content[(atom_offset - 4) as usize..atom_offset as usize]
                        .try_into()
                        .unwrap(),
                );

                atom.primaries = u16::from_be_bytes(
                    file_content[(atom_offset + 8) as usize..(atom_offset + 10) as usize]
                        .try_into()
                        .unwrap(),
                );

                atom.transfer_function = u16::from_be_bytes(
                    file_content[(atom_offset + 10) as usize..(atom_offset + 12) as usize]
                        .try_into()
                        .unwrap(),
                );

                atom.matrix = u16::from_be_bytes(
                    file_content[(atom_offset + 12) as usize..(atom_offset + 14) as usize]
                        .try_into()
                        .unwrap(),
                );

                return Ok(atom);
            }

            let align = file_content.len() % pattern.len();
            file_content.drain(..file_content.len() - align);

            offset += bytes_read as u64;
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
            println!("File opened");
            file
        }
        Err(e) => panic!("An error occurred when open file: {}", e),
    };

    match ColrAtom::search(&mut stream, &COLR_ATOM) {
        Ok(atom) => {
            println!("Found atom: {:?}", atom);
        }
        Err(e) => {
            println!("An error occurred: {}", e);
        }
    };
}