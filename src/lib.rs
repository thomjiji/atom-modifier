use aho_corasick::AhoCorasick;
use clap::Parser;
use std::fs::{File, OpenOptions};
use std::io::{self, Seek, Write};
use std::io::{Error, Read};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long = "input-file-path", value_name = "FILE")]
    pub input_file_path: String,

    #[arg(short, long = "show-property")]
    show_property: Option<String>,

    #[arg(short, long = "primary", value_name = "INDEX_VALUE")]
    /// Change the "color primaries index" to <INDEX_VALUE>
    pub primary_index: u8,

    #[arg(short, long = "transfer-function", value_name = "INDEX_VALUE")]
    /// Change the "transfer characteristics index" to <INDEX_VALUE>
    pub transfer_function_index: u8,

    #[arg(short, long = "matrix", value_name = "INDEX_VALUE")]
    /// Change the "matrix coeffients index" to <INDEX_VALUE>
    pub matrix_index: u8,

    /// Change the Gamma value to <GAMA_VALUE> if gama atom present
    #[arg(short, long = "gama-value", default_value_t = -1.0)]
    pub gama_value: f32,
}

static COLR_ATOM_HEADER: [u8; 4] = [0x63, 0x6f, 0x6c, 0x72]; // "colr"
static GAMA_ATOM_HEADER: [u8; 4] = [0x67, 0x61, 0x6d, 0x61]; // "gama"
static FRAME_HEADER: [u8; 4] = [0x69, 0x63, 0x70, 0x66]; // "icpf"

#[derive(Default, Debug)]
enum ColorParameterType {
    #[default]
    Nclc, // for video
    _Prof, // for print
    _Unknown,
}

#[derive(Default, Debug)]
struct ColrAtom {
    size: u32,
    _color_parameter_type: ColorParameterType,
    offset: u64,
    primary_index: u16,
    transfer_function_index: u16,
    matrix_index: u16,
    matched: bool,
}

#[derive(Default, Debug)]
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

#[derive(Default, Debug)]
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

#[derive(Default, Debug)]
pub struct Video {
    colr_atom: ColrAtom,
    gama_atom: GamaAtom,
    frames: Vec<ProResFrame>,
    frame_count: i64,
}

impl Video {
    pub fn read_file(
        file_path: &str,
        read: Option<bool>,
        write: Option<bool>,
    ) -> Result<File, Error> {
        let read_permission = read.unwrap_or(true);
        let write_permission = write.unwrap_or(false);

        let file = OpenOptions::new()
            .read(read_permission)
            .write(write_permission)
            .open(file_path)?;

        Ok(file)
    }

    pub fn decode(&mut self, file_path: &str) -> Result<(), Error> {
        let file = OpenOptions::new()
            .read(true)
            .open(file_path)
            .expect("Some issue occur when reading file.");

        let search_patterns = [COLR_ATOM_HEADER, GAMA_ATOM_HEADER, FRAME_HEADER];
        let ac = AhoCorasick::new(search_patterns).unwrap();

        for mat in ac.stream_find_iter(&file) {
            let mut file_to_seek = OpenOptions::new()
                .read(true)
                .open(file_path)
                .expect("Some issue occur when reading file.");

            match mat {
                Ok(mat) => match mat.pattern().as_u32() {
                    0 => self.construct_colr_atom(&mut file_to_seek, mat.start() - 4)?,
                    1 => self.construct_gama_atom(&mut file_to_seek, mat.start() - 4)?,
                    2 => self.construct_prores_frame(&mut file_to_seek, mat.start() - 4)?,
                    _ => unreachable!(),
                },
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    fn construct_colr_atom(&mut self, file: &mut File, offset: usize) -> Result<(), Error> {
        self.colr_atom.offset = offset as u64;

        let mut size_buf = [0; 4];
        file.seek(io::SeekFrom::Start(self.colr_atom.offset))?;
        file.read_exact(&mut size_buf)?;
        self.colr_atom.size = u32::from_be_bytes(size_buf);

        let mut nclc_buf = [0; 2];
        file.seek(io::SeekFrom::Start(self.colr_atom.offset + 12))?;
        file.read_exact(&mut nclc_buf)?;
        self.colr_atom.primary_index = u16::from_be_bytes(nclc_buf);

        file.seek(io::SeekFrom::Start(self.colr_atom.offset + 14))?;
        file.read_exact(&mut nclc_buf)?;
        self.colr_atom.transfer_function_index = u16::from_be_bytes(nclc_buf);

        file.seek(io::SeekFrom::Start(self.colr_atom.offset + 16))?;
        file.read_exact(&mut nclc_buf)?;
        self.colr_atom.matrix_index = u16::from_be_bytes(nclc_buf);

        self.colr_atom.matched = true;

        Ok(())
    }

    fn construct_gama_atom(&mut self, file: &mut File, offset: usize) -> Result<(), Error> {
        self.gama_atom.offsets.push(offset as u64);

        let mut size_buf = [0; 4];
        file.seek(io::SeekFrom::Start(offset as u64))?;
        file.read_exact(&mut size_buf)?;

        if size_buf == [0x00, 0x00, 0x00, 0x0c] {
            self.gama_atom.the_actual_gama_offset = offset as u64;
            self.gama_atom.size = u32::from_be_bytes(size_buf);
            self.gama_atom.matched = true;

            let mut value_buf = size_buf;
            file.seek(io::SeekFrom::Start(offset as u64 + 8))?;
            file.read_exact(&mut value_buf)?;
            self.gama_atom.gama_value = u32::from_be_bytes(value_buf);
        }

        Ok(())
    }

    fn construct_prores_frame(&mut self, file: &mut File, offset: usize) -> Result<(), Error> {
        let mut frame = ProResFrame::new();
        frame.offset = offset as u64;

        let mut frame_size_buf = [0; 4];
        file.seek(io::SeekFrom::Start(frame.offset))?;
        file.read_exact(&mut frame_size_buf)?;
        frame.frame_size = u32::from_be_bytes(frame_size_buf);

        let mut frame_header_size_buf = [0; 2];
        file.seek(io::SeekFrom::Start(frame.offset + 8))?;
        file.read_exact(&mut frame_header_size_buf)?;
        frame.frame_header_size = u16::from_be_bytes(frame_header_size_buf);

        let mut color_primaries_buf = [0; 1];
        file.seek(io::SeekFrom::Start(frame.offset + 22))?;
        file.read_exact(&mut color_primaries_buf)?;
        frame.color_primaries = u8::from_be_bytes(color_primaries_buf);

        let mut transfer_characteristics_buf = [0; 1];
        file.seek(io::SeekFrom::Start(frame.offset + 23))?;
        file.read_exact(&mut transfer_characteristics_buf)?;
        frame.transfer_characteristic = u8::from_be_bytes(transfer_characteristics_buf);

        let mut matrix_coefficients_buf = [0; 1];
        file.seek(io::SeekFrom::Start(frame.offset + 24))?;
        file.read_exact(&mut matrix_coefficients_buf)?;
        frame.matrix_coefficients = u8::from_be_bytes(matrix_coefficients_buf);

        self.frames.push(frame);
        self.frame_count += 1;

        Ok(())
    }

    pub fn encode(
        &self,
        file: &mut File,
        video: &Video,
        target_color_primaries: u8,
        target_transfer_functions: u8,
        target_matrix: u8,
        target_gama_value: f32,
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

        // Overwrite gama atom
        //
        // If gama atom matched, it means that the original file has gama atom and also
        // has gama value. At this time, -g of args can work, and the original gama
        // value is overwritten with the value given by -g. If gama atom doesn't match,
        // just ignore the -g arg.
        if self.gama_atom.matched
            && self.gama_atom.the_actual_gama_offset != 0
            && target_gama_value != -1.0
        {
            let new_gama_value = Self::float_to_fixed_point_bytes(target_gama_value);
            file.seek(io::SeekFrom::Start(
                video.gama_atom.the_actual_gama_offset + 8,
            ))?;
            file.write_all(&new_gama_value)?;
        }

        Ok(())
    }

    fn float_to_fixed_point_bytes(input_gama_value: f32) -> [u8; 4] {
        let fixed_value = (input_gama_value * 65536_f32) as i32;
        fixed_value.to_be_bytes()
    }

    fn _fixed_point_hex_to_float(input_gama_value: u32) -> f64 {
        input_gama_value as f64 / 65536_f64
    }
}
