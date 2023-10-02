use clap::Parser;
use std::fs::{File, OpenOptions};
use std::io::{self, Seek, Write};
use std::io::{Error, Read};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_file_path: String,

    #[arg(short, long)]
    show_property: Option<String>,

    #[arg(short, long)]
    primaries: String,

    #[arg(short, long)]
    transfer_function: String,

    #[arg(short, long)]
    matrix: String,

    #[arg(short, long, default_value_t = String::from("0"))]
    gama: String,
}

static COLR_ATOM_HEADER: [u8; 4] = [0x63, 0x6f, 0x6c, 0x72]; // "colr"
static GAMA_ATOM_HEADER: [u8; 4] = [0x67, 0x61, 0x6d, 0x61]; // "gama"
static FRAME_HEADER: [u8; 4] = [0x69, 0x63, 0x70, 0x66]; // "icpf"

#[derive(Debug)]
enum ColorParameterType {
    Nclc, // for video
    Prof, // for print
    Unknown,
}

trait AtomTrait {
    fn find_pattern_position(buffer: &[u8], pattern: &[u8]) -> Option<usize> {
        buffer
            .windows(pattern.len())
            .position(|window| window == pattern)
    }
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

        let pattern_position = Self::find_pattern_position(&buffer, pattern);

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

    fn overwrite(self, file: &mut File, target_colr_tags: &[u8; 3]) -> Result<(), Error> {
        let primary = target_colr_tags[0];
        let transfer_function = target_colr_tags[1];
        let matrix = target_colr_tags[2];

        let buf = &[0, primary, 0, transfer_function, 0, matrix];
        file.seek(io::SeekFrom::Start(self.offset + 12))?;
        file.write_all(buf)?;

        file.sync_all().expect("file sync all has some problem");

        Ok(())
    }
}

impl AtomTrait for ColrAtom {}

struct GamaAtom {
    size: u8,
    data: u32,
    offset: u64,
}

impl AtomTrait for GamaAtom {
    fn find_pattern_position(buffer: &[u8], pattern: &[u8]) -> Option<usize> {
        todo!()
    }
}

struct ProResFrame {
    offset: u64,
    frame_size: u32,
    frame_id: f32,
    frame_header: ProResFrameHeader,
}

struct ProResFrameHeader {
    offset: u64,
    frame_header_size: u16,
    color_primaries: u8,
    transfer_characteristic: u8,
    matrix_coefficients: u8,
}

fn main() {
    let args = Args::parse();

    // Open file stream
    let mut stream = match OpenOptions::new()
        .read(true)
        .write(true)
        .open(args.input_file_path)
    {
        Ok(file) => {
            println!("File opened...");
            file
        }
        Err(e) => panic!("An error occurred when open file: {}", e),
    };

    // Fetch colr atom information from file stream.
    let start = Instant::now();
    let colr_atom_found = match ColrAtom::search(&mut stream, &COLR_ATOM_HEADER) {
        Ok(atom) => {
            println!("Found atom: \n\t{:?}", atom);
            Some(atom)
        }
        Err(e) => {
            println!("An error occurred: {}", e);
            None
        }
    };
    let duration = start.elapsed();
    println!(
        "Time elapsed in this search implementation is: {:?}",
        duration
    );

    // Overwrite colr atom
    let primaries = match args.primaries.parse::<u8>() {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Error: The provided value for primaries is not a valid integer in the range of 0 to 255");
            return;
        }
    };
    let transfer_function = match args.transfer_function.parse::<u8>() {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Error: The provided value for transfer_function is not a valid integer in the range of 0 to 255");
            return;
        }
    };
    let matrix = match args.matrix.parse::<u8>() {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Error: The provided value for matrix is not a valid integer in the range of 0 to 255");
            return;
        }
    };

    let colr_target: [u8; 3] = [primaries, transfer_function, matrix];

    let colr_atom_found = colr_atom_found.unwrap();

    colr_atom_found
        .overwrite(&mut stream, &colr_target)
        .expect("Something bad happened when overwrite colr atom.");
}
