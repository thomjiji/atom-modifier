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
struct Video {
    colr_atom: ColrAtom,
    gama_atom: GamaAtom,
    frames: Vec<ProResFrame>,
    frame_count: i64,
}

impl Video {
    pub fn new() -> Self {
        Self {
            colr_atom: ColrAtom::new(),
            gama_atom: GamaAtom::new(),
            frames: Vec::new(),
            frame_count: 0,
        }
    }

    fn read_file(file_path: &str, read: Option<bool>, write: Option<bool>) -> Result<File, Error> {
        let read_permission = read.unwrap_or(true);
        let write_permission = write.unwrap_or(false);

        let file = OpenOptions::new()
            .read(read_permission)
            .write(write_permission)
            .open(file_path)?;

        Ok(file)
    }

    fn decode(file: &mut File) -> Result<Self, Error> {
        let mut video = Video::new();

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        video.colr_atom.search(buffer.as_slice());

        // Handle colr atom
        if video.colr_atom.matched {
            video.colr_atom.size = u32::from_be_bytes(
                buffer[video.colr_atom.offset as usize..(video.colr_atom.offset + 4) as usize]
                    .try_into()
                    .unwrap(),
            );

            video.colr_atom.primaries = u16::from_be_bytes(
                buffer[(video.colr_atom.offset + 12) as usize
                    ..(video.colr_atom.offset + 14) as usize]
                    .try_into()
                    .unwrap(),
            );

            video.colr_atom.transfer_function = u16::from_be_bytes(
                buffer[(video.colr_atom.offset + 14) as usize
                    ..(video.colr_atom.offset + 16) as usize]
                    .try_into()
                    .unwrap(),
            );

            video.colr_atom.matrix = u16::from_be_bytes(
                buffer[(video.colr_atom.offset + 16) as usize
                    ..(video.colr_atom.offset + 18) as usize]
                    .try_into()
                    .unwrap(),
            );
        }

        // Handle each frame
        loop {
            let mut frame = ProResFrame::new();
            match frame.search(buffer.as_slice()) {
                Some(offset) => {
                    if video.frames.is_empty() {
                        frame.offset = (offset - 4) as u64;
                    } else {
                        let last_frame = video
                            .frames
                            .last()
                            .expect("Unexpectedly, no last frame was found.");
                        // next pos/offset = previous pos/offset + previous frame size
                        frame.offset = last_frame.offset + last_frame.frame_size as u64;
                    }

                    frame.frame_size =
                        u32::from_be_bytes(buffer[offset - 4..offset].try_into().unwrap());

                    frame.frame_header_size =
                        u16::from_be_bytes(buffer[offset + 4..offset + 6].try_into().unwrap());

                    frame.color_primaries =
                        u8::from_be_bytes(buffer[offset + 18..offset + 19].try_into().unwrap());

                    frame.transfer_characteristics =
                        u8::from_be_bytes(buffer[offset + 19..offset + 20].try_into().unwrap());

                    frame.matrix_coefficients =
                        u8::from_be_bytes(buffer[offset + 20..offset + 21].try_into().unwrap());

                    video.frames.push(frame);
                    video.frame_count += 1;
                }
                None => {
                    if video.frames.is_empty() {
                        println!("No ProRes frame was found in the file.");
                        break;
                    } else if buffer.len() < video.frames.last().unwrap().frame_size as usize {
                        println!("Reach the end of the file stream.");
                        break;
                    } else {
                        // Temporary solution for non-icpf frame: set the frame_id to -1.0.
                        frame.frame_id = -1.0;
                    }
                }
            };

            let previous_frame_size = video.frames.last().unwrap().frame_size;
            buffer = buffer.split_off(previous_frame_size as usize);
        }

        Ok(video)
    }
}

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
    matched: bool,
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
            matched: false,
        }
    }

    fn search(&mut self, file: &mut File, pattern: &[u8]) -> Result<Self, Error> {
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let pattern_position = self.find_pattern_position(&buffer);

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

    fn overwrite(&self, file: &mut File, target_colr_tags: &[u8; 3]) -> Result<(), Error> {
        let primary = target_colr_tags[0];
        let transfer_function = target_colr_tags[1];
        let matrix = target_colr_tags[2];

        let buf = &[0, primary, 0, transfer_function, 0, matrix];
        file.seek(io::SeekFrom::Start(self.offset + 12))?;
        file.write_all(buf)?;

        file.sync_all().expect("file sync all has some problem");

        Ok(())
    }

    fn search(&mut self, buffer: &[u8]) -> Option<usize> {
        match buffer
            .windows(COLR_ATOM_HEADER.len())
            .position(|window| window == COLR_ATOM_HEADER)
        {
            Some(offset) => {
                // Set the offset to the position on the pattern match 4 bytes forward.
                self.offset = (offset - 4) as u64;
                self.matched = true;
                Some(offset - 4)
            }
            None => {
                println!("colr atom header was not found in the buffer (file stream).");
                None
            }
        }
    }
}

#[derive(Debug)]
struct GamaAtom {
    size: u32,
    gama_value: u32,
    offsets: Vec<u64>,
    the_actual_gama_offset: u64,
    matched: bool,
}

impl GamaAtom {
    fn new() -> GamaAtom {
        Self {
            size: 0,
            gama_value: 0,
            offsets: Vec::new(),
            the_actual_gama_offset: 0,
            matched: false,
        }
    }

    fn search(&mut self, buffer: &[u8], colr_atom_transfer_function: u16) {
        for (i, window) in buffer.windows(GAMA_ATOM_HEADER.len()).enumerate() {
            if window == GAMA_ATOM_HEADER {
                let offset = (i - 4) as u64;
                self.offsets.push(offset);
                self.matched = true;
            }
        }

        match self.offsets.len() {
            0 => {
                println!("gama atom header was not found in the buffer (file stream).");
            }
            1 => {
                if colr_atom_transfer_function == 1 {
                    eprintln!("There is not supposed to have gama atom pattern.")
                } else {
                    self.the_actual_gama_offset = self.offsets[1];
                }
            }
            2 => {
                if let Some(last) = self.offsets.last() {
                    self.the_actual_gama_offset += last;
                }
            }
            _ => {
                eprintln!("There are more than 2 matches for gama atom header, strange! Please investigate.");
            }
        }
    }
}

#[derive(Debug)]
struct ProResFrame {
    offset: u64,
    frame_size: u32,
    frame_id: f32, // if the value of it is -1.0, it means it's not a icpf frame.
    frame_header_size: u16,
    color_primaries: u8,
    transfer_characteristics: u8,
    matrix_coefficients: u8,
}

impl ProResFrame {
    fn new() -> Self {
        Self {
            offset: 0,
            frame_size: 0,
            frame_id: 0.0,
            frame_header_size: 0,
            color_primaries: 0,
            transfer_characteristics: 0,
            matrix_coefficients: 0,
        }
    }

    fn search(&mut self, buffer: &[u8]) -> Option<usize> {
        buffer
            .windows(FRAME_HEADER.len())
            .position(|window| window == FRAME_HEADER)
    }
}

fn main() {
    let args = Args::parse();

    let mut file = match Video::read_file(args.input_file_path.as_str(), Some(true), Some(true)) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };
    let video = Video::decode(&mut file);
    println!("{:#?}", video);
}
