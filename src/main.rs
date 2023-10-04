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
    primary_index: u8,

    #[arg(short, long)]
    transfer_function_index: u8,

    #[arg(short, long)]
    matrix_index: u8,

    #[arg(short, long, default_value_t = 0)]
    gama_value: u32,
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
        let file = Self::read_file(file_path, Some(true), Some(false))
            .expect("Some issue occur when reading file.");

        let mut video = Video::new();

        let search_patterns = [COLR_ATOM_HEADER, GAMA_ATOM_HEADER, FRAME_HEADER];
        let ac = AhoCorasick::new(search_patterns).unwrap();

        for mat in ac.stream_find_iter(&file) {
            let mut file_to_seek = Self::read_file(file_path, Some(true), Some(false)).unwrap();

            match mat {
                Ok(mat) => match mat.pattern().as_u32() {
                    // Construct colr atom
                    0 => {
                        video.colr_atom.offset = (mat.start() - 4) as u64;

                        let mut size_buf = [0; 4];
                        file_to_seek.seek(io::SeekFrom::Start(video.colr_atom.offset))?;
                        file_to_seek.read_exact(&mut size_buf)?;
                        video.colr_atom.size = u32::from_be_bytes(size_buf);

                        let mut nclc_buf = [0; 2];
                        file_to_seek.seek(io::SeekFrom::Start(video.colr_atom.offset + 12))?;
                        file_to_seek.read_exact(&mut nclc_buf)?;
                        video.colr_atom.primary_index = u16::from_be_bytes(nclc_buf);

                        file_to_seek.seek(io::SeekFrom::Start(video.colr_atom.offset + 14))?;
                        file_to_seek.read_exact(&mut nclc_buf)?;
                        video.colr_atom.transfer_function_index = u16::from_be_bytes(nclc_buf);

                        file_to_seek.seek(io::SeekFrom::Start(video.colr_atom.offset + 16))?;
                        file_to_seek.read_exact(&mut nclc_buf)?;
                        video.colr_atom.matrix_index = u16::from_be_bytes(nclc_buf);

                        video.colr_atom.matched = true;
                    }
                    // Construct gama atom
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
                    // Construct each ProRes frame
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
                        frame.transfer_characteristic =
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
                // An error is yielded if there was a problem reading from the reader given.
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
        video: &Video,
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

        file.sync_all().expect("File.sync_all() has some problem.");

        // Overwrite each ProRes frame
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
    Nclc,  // for video
    _Prof, // for print
    _Unknown,
}

#[derive(Debug)]
struct ColrAtom {
    size: u32,
    _color_parameter_type: ColorParameterType,
    offset: u64,
    primary_index: u16,
    transfer_function_index: u16,
    matrix_index: u16,
    matched: bool,
}

impl ColrAtom {
    fn new() -> ColrAtom {
        Self {
            size: 0,
            _color_parameter_type: ColorParameterType::Nclc,
            offset: 0,
            primary_index: 0,
            transfer_function_index: 0,
            matrix_index: 0,
            matched: false,
        }
    }
}

#[derive(Debug)]
struct GamaAtom {
    size: u32,
    // The actual gama value: for example 2.4, 2.2, etc (It looks like this in
    // hexadecimal form: 0x00, 0x02, 0x66, 0x66).
    gama_value: u32,
    // gama atom candidates
    offsets: Vec<u64>,
    // What is the real gama offset? As long as the four bytes before the gama
    // offset/position candicate are in a specific pattern, i.e. like this: 0x00, 0x00,
    // 0x00, 0x0c (It indicates the size of the gama atom, 12).
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
}

#[derive(Debug)]
struct ProResFrame {
    offset: u64,
    frame_size: u32,
    _frame_id: f32, // if the value of it is -1.0, it means it's not a icpf frame.
    frame_header_size: u16,
    color_primaries: u8,
    transfer_characteristic: u8,
    matrix_coefficients: u8,
}

impl ProResFrame {
    fn new() -> Self {
        Self {
            offset: 0,
            frame_size: 0,
            _frame_id: 0.0,
            frame_header_size: 0,
            color_primaries: 0,
            transfer_characteristic: 0,
            matrix_coefficients: 0,
        }
    }
}

fn main() {
    let args = Args::parse();

    let now = Instant::now();
    let video = Video::decode(args.input_file_path.as_str()).unwrap_or_else(|e| {
        eprintln!(
            "Error decoding (constructing colr, gama atom and each frames) video: {}",
            e
        );
        std::process::exit(1);
    });
    println!(
        "- Time elapsed after decoding the file: {:?}",
        now.elapsed()
    );

    let mut file = match Video::read_file(args.input_file_path.as_str(), Some(true), Some(true)) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error reading input file: {}", e);
            std::process::exit(1);
        }
    };

    let now = Instant::now();
    Video::encode(
        &mut file,
        &video,
        args.primary_index,
        args.transfer_function_index,
        args.matrix_index,
    )
    .expect("Encode has some problem.");
    println!(
        "- Time elapsed after encoding the file: {:?}",
        now.elapsed()
    );

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("output.txt")
        .unwrap();
    writeln!(file, "{:#?}", video).unwrap();
    // println!("{:#?}", video);
}
