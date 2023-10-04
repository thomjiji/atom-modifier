use aho_corasick::AhoCorasick;
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

    fn decode(file_path: &str) -> Result<Self, Error> {
        let mut file = Self::read_file(file_path, Some(true), Some(true)).unwrap();

        let mut video = Video::new();

        let search_patterns = [COLR_ATOM_HEADER, GAMA_ATOM_HEADER, FRAME_HEADER];
        let ac = AhoCorasick::new(search_patterns).unwrap();

        for mat in ac.stream_find_iter(&file) {
            let mut file_to_seek = Self::read_file(file_path, Some(true), Some(true)).unwrap();

            match mat {
                Ok(mat) => match mat.pattern().as_u32() {
                    0 => {
                        video.colr_atom.offset = (mat.start() - 4) as u64;

                        let mut size_buf = [0; 4];
                        file_to_seek.seek(io::SeekFrom::Start(video.colr_atom.offset))?;
                        file_to_seek.read_exact(&mut size_buf)?;
                        video.colr_atom.size = u32::from_be_bytes(size_buf);

                        let mut nclc_buf = [0; 2];
                        file_to_seek.seek(io::SeekFrom::Start(video.colr_atom.offset + 12))?;
                        file_to_seek.read_exact(&mut nclc_buf)?;
                        video.colr_atom.primaries = u16::from_be_bytes(nclc_buf);

                        file_to_seek.seek(io::SeekFrom::Start(video.colr_atom.offset + 14))?;
                        file_to_seek.read_exact(&mut nclc_buf)?;
                        video.colr_atom.transfer_function = u16::from_be_bytes(nclc_buf);

                        file_to_seek.seek(io::SeekFrom::Start(video.colr_atom.offset + 16))?;
                        file_to_seek.read_exact(&mut nclc_buf)?;
                        video.colr_atom.matrix = u16::from_be_bytes(nclc_buf);

                        video.colr_atom.matched = true;
                    }
                    1 => {
                        let offset = (mat.start() - 4) as u64;
                        video.gama_atom.offsets.push(offset);

                        let mut size_buf = [0; 4];
                        file_to_seek.seek(io::SeekFrom::Start(offset))?;
                        file_to_seek.read_exact(&mut size_buf)?;

                        if size_buf == [0x00, 0x00, 0x00, 0x0c] {
                            video.gama_atom.the_actual_gama_offset = offset;
                            video.gama_atom.size = u32::from_be_bytes(size_buf);
                            video.gama_atom.matched = true;

                            let mut value_buf = size_buf;
                            file_to_seek.seek(io::SeekFrom::Start(offset + 8))?;
                            file_to_seek.read_exact(&mut value_buf)?;
                            video.gama_atom.gama_value = u32::from_be_bytes(value_buf);
                        }
                    }
                    2 => {
                        let mut frame = ProResFrame::new();
                        frame.offset = (mat.start() - 4) as u64;

                        let mut frame_size_buf = [0; 4];
                        file_to_seek.seek(io::SeekFrom::Start(frame.offset))?;
                        file_to_seek.read_exact(&mut frame_size_buf)?;
                        frame.frame_size = u32::from_be_bytes(frame_size_buf);

                        let mut frame_header_size_buf = [0; 2];
                        file_to_seek.seek(io::SeekFrom::Start(frame.offset + 8))?;
                        file_to_seek.read_exact(&mut frame_header_size_buf)?;
                        frame.frame_header_size = u16::from_be_bytes(frame_header_size_buf);

                        let mut color_primaries_buf = [0; 1];
                        file_to_seek.seek(io::SeekFrom::Start(frame.offset + 22))?;
                        file_to_seek.read_exact(&mut color_primaries_buf)?;
                        frame.color_primaries = u8::from_be_bytes(color_primaries_buf);

                        let mut transfer_characteristics_buf = [0; 1];
                        file_to_seek.seek(io::SeekFrom::Start(frame.offset + 23))?;
                        file_to_seek.read_exact(&mut transfer_characteristics_buf)?;
                        frame.transfer_characteristics =
                            u8::from_be_bytes(transfer_characteristics_buf);

                        let mut matrix_coefficients_buf = [0; 1];
                        file_to_seek.seek(io::SeekFrom::Start(frame.offset + 24))?;
                        file_to_seek.read_exact(&mut matrix_coefficients_buf)?;
                        frame.matrix_coefficients = u8::from_be_bytes(matrix_coefficients_buf);

                        video.frames.push(frame);
                        video.frame_count += 1;
                    }
                    _ => unreachable!(),
                },
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(video)
    }

    fn encode(
        file: &mut File,
        video: Video,
        target_color_primaries: u8,
        target_transfer_functions: u8,
        target_matrix: u8,
    ) -> io::Result<()> {
        // Overwrite mov colr atom
        let buf = [
            0,
            target_color_primaries,
            0,
            target_transfer_functions,
            0,
            target_matrix,
        ];
        file.seek(io::SeekFrom::Start(video.colr_atom.offset + 12))?;
        file.write_all(&buf)?;

        file.sync_all().expect("file sync all has some problem");

        for frame in video.frames.iter() {
            let buf = [
                target_color_primaries,
                target_transfer_functions,
                target_matrix,
            ];
            file.seek(io::SeekFrom::Start(frame.offset + 22))?;
            file.write_all(&buf)?;
        }

        Ok(())
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

    fn build(&mut self, file: &mut File) -> io::Result<()> {
        let mut buf: [u8; 4] = [0; 4];
        file.seek(io::SeekFrom::Start(self.offset))?;
        file.read_exact(&mut buf);
        self.frame_size = u32::from_be_bytes(buf);
        Ok(())
    }
}

fn main() {
    let args = Args::parse();

    let now = Instant::now();
    let video = Video::decode(args.input_file_path.as_str()).unwrap();
    println!("- Time elapsed: {:?}", now.elapsed());

    let mut file = OpenOptions::new()
        .write(true)
        .open("output.txt").unwrap();

    writeln!(file, "{:#?}", video).unwrap();
}
