use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, Write};

use aho_corasick::AhoCorasick;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Modify color primaries, transfer characteristics, matrix coefficients, and gamma value of QuickTime file.",
    long_about = "This program allows you to modify the color primaries, transfer characteristics, matrix coefficients, and gamma value of QuickTime file. Before do the modification, it will create a backup of the input file."
)]
pub struct Args {
    #[arg(short, long = "input-file-path", value_name = "FILE")]
    /// The path to the input file
    pub input_file_path: String,

    #[arg(short, long = "color-primaries", value_name = "INDEX_VALUE")]
    /// Change the "color primaries index" to <INDEX_VALUE>
    pub primary_index: u8,

    #[arg(short, long = "transfer-characteristics", value_name = "INDEX_VALUE")]
    /// Change the "transfer characteristics index" to <INDEX_VALUE>
    pub transfer_function_index: u8,

    #[arg(short, long = "matrix-coefficients", value_name = "INDEX_VALUE")]
    /// Change the "matrix coefficients index" to <INDEX_VALUE>
    pub matrix_index: u8,

    #[arg(short, long = "gama-value", default_value_t = -1.0)]
    /// The gamma value to set. If not present, defaults to -1.0
    pub gama_value: f32,
}

static COLR_ATOM_HEADER: [u8; 4] = [0x63, 0x6f, 0x6c, 0x72]; // "colr"
static GAMA_ATOM_HEADER: [u8; 4] = [0x67, 0x61, 0x6d, 0x61]; // "gama"
static PRORES_FRAME_HEADER: [u8; 4] = [0x69, 0x63, 0x70, 0x66]; // "icpf"

#[derive(Default, Debug, PartialEq)]
enum ColorParameterType {
    #[default]
    Nclc, // for video
    _Prof, // for print
    _Unknown,
}

#[derive(Default, Debug, PartialEq)]
struct ColrAtom {
    size: u32,
    offset: u64,
    _color_parameter_type: ColorParameterType,
    primary_index: u16,
    transfer_function_index: u16,
    matrix_index: u16,
    matched: bool,
}

#[derive(Default, Debug, PartialEq)]
struct GamaAtom {
    size: u32,
    // gama atom candidates
    offsets: Vec<u64>,
    // What is the real gama offset? As long as the four bytes before the gama
    // offset/position candidates are in a specific pattern, i.e. like this: 0x00, 0x00,
    // 0x00, 0x0c (It indicates the size of the gama atom, 12).
    the_actual_gama_offset: u64,
    // The actual gama value: for example 2.4, 2.2, etc (It looks like this in
    // hexadecimal form: 0x00, 0x02, 0x66, 0x66).
    gama_value: u32,
    matched: bool,
}

#[derive(Default, Debug, Clone, PartialEq)]
struct ProResFrame {
    frame_size: u32,
    offset: u64,
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

#[derive(Default, Debug, PartialEq)]
pub struct Video {
    colr_atom: ColrAtom,
    gama_atom: GamaAtom,
    frames: Vec<ProResFrame>,
    frame_count: i64,
}

impl Video {
    /// Constructs a colr atom and sets its offset, size, primary index, transfer
    /// function index, and matrix index.
    ///
    /// # Arguments
    ///
    /// * `file` - A mutable reference to a `File` object.
    /// * `offset` - The offset of the colr atom as a `usize`.
    ///
    /// # Returns
    ///
    /// An `io::Result` indicating whether the operation was successful or not.
    fn construct_colr_atom(&mut self, file: &mut File, offset: usize) -> io::Result<()> {
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

    /// Constructs a gama atom.
    ///
    /// Based on whether the four bytes before the gama
    /// offset/position candidates are in a specific pattern (that is: `0x00, 0x00,
    /// 0x00, 0x0c` which is the size of gama atom) to determine whether it's a real
    /// offset of gama atom.
    ///
    /// # Arguments
    ///
    /// * `file` - A mutable reference to a `File` instance.
    /// * `offset` - The offset of the gama atom in the file.
    ///
    /// # Errors
    ///
    /// This function returns an `io::Result` in case of an I/O error occurring when
    /// seeking file or read bytes from file..
    fn construct_gama_atom(&mut self, file: &mut File, offset: usize) -> io::Result<()> {
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

    /// Constructs a ProRes frame from a file at a given offset.
    ///
    /// # Arguments
    ///
    /// * `file` - A mutable reference to a `File` object.
    /// * `offset` - The offset in bytes from the start of the file where the frame is
    ///   located.
    ///
    /// # Errors
    ///
    /// This function returns an `io::Result` in case of any I/O errors that occur while
    /// reading from the file.
    fn construct_prores_frame(&mut self, file: &mut File, offset: usize) -> io::Result<()> {
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

    /// Decodes a video file and constructs the corresponding atoms and frames.
    ///
    /// # Arguments
    ///
    /// * `file_path` - A string slice that holds the path to the video file.
    ///
    /// # Examples
    ///
    /// ```
    /// use atom_modifier::Video;
    ///
    /// let mut video = Video::default();
    /// video.decode("tests/footages/1-1-1_2frames_prores422.mov").unwrap();
    /// ```
    ///
    /// # Notes
    ///
    /// This method only needs read access to the file.
    pub fn decode(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = OpenOptions::new().read(true).open(file_path)?;

        let search_patterns = [COLR_ATOM_HEADER, GAMA_ATOM_HEADER, PRORES_FRAME_HEADER];
        let ac = AhoCorasick::new(search_patterns)?;

        // error of result is std::io::Error
        for result in ac.stream_find_iter(&file) {
            let mat = result?;

            let mut file_to_seek = OpenOptions::new().read(true).open(file_path)?;

            match mat.pattern().as_u32() {
                0 => self.construct_colr_atom(&mut file_to_seek, mat.start() - 4)?,
                1 => self.construct_gama_atom(&mut file_to_seek, mat.start() - 4)?,
                2 => self.construct_prores_frame(&mut file_to_seek, mat.start() - 4)?,
                _ => unreachable!(),
            };
        }

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
            let new_gama_value = Self::float_to_bytes(target_gama_value);
            file.seek(io::SeekFrom::Start(
                video.gama_atom.the_actual_gama_offset + 8,
            ))?;
            file.write_all(&new_gama_value)?;
        }

        Ok(())
    }

    /// Converts a floating point number to a byte array.
    ///
    /// This function takes a 32-bit floating point number, converts it to a fixed-point
    /// number with 16 fractional bits, then changes the fixed-point number to a
    /// big-endian i32, and finally, represents the i32 as a byte array.
    ///
    /// # Arguments
    ///
    /// * `input_gama_value` - A 32-bit floating point number to be converted.
    ///
    /// # Returns
    ///
    /// A 4 bytes array which represents the input floating point number.
    ///
    /// # Example
    ///
    /// ```
    /// use crate::atom_modifier::Video;
    ///
    /// let float_value = 2.2;
    /// let bytes = Video::float_to_bytes(float_value);
    /// assert_eq!(bytes, [0x00, 0x02, 0x33, 0x33]);
    /// ```
    pub fn float_to_bytes(input_gama_value: f32) -> [u8; 4] {
        // Left shift 1 by 16 bits. This is equivalent to multiplying by 2^16 (1 * 2^16).
        let fixed_value = (input_gama_value * (1 << 16) as f32) as i32;
        fixed_value.to_be_bytes()
    }

    /// Converts a byte array to a floating point number.
    ///
    /// This function takes a 4-byte array, interprets it as a big-endian i32, then
    /// interprets that i32 as a fixed-point number with 16 fractional bits, and finally
    /// converts that to a 32-bit floating point number.
    ///
    /// # Arguments
    ///
    /// * `bytes` - An array of 4 bytes which represent a big-endian i32 and
    /// subsequently a fixed-point number with 16 fractional bits.
    ///
    /// # Returns
    ///
    /// A 32-bit floating point number representation of the input byte array.
    ///
    /// # Example
    ///
    /// ```
    /// use crate::atom_modifier::Video;
    ///
    /// let bytes = [0x00, 0x02, 0x66, 0x66];
    /// let float_value = Video::bytes_to_float(bytes);
    /// assert_eq!(float_value, 2.4);
    /// ```
    pub fn bytes_to_float(bytes: [u8; 4]) -> f32 {
        let fixed_value = i32::from_be_bytes(bytes);
        // TODO: deal with floating point rounding errors here.
        (fixed_value as f32 / (1 << 16) as f32).round_to_decimals(2)
    }
}

trait RoundTo {
    fn round_to_decimals(&self, num: i32) -> f32;
}

impl RoundTo for f32 {
    fn round_to_decimals(&self, num: i32) -> f32 {
        let multiplier = 10f32.powi(num);
        (self * multiplier).round() / multiplier
    }
}

#[cfg(test)]
mod tests {
    use crate::ColorParameterType::Nclc;

    use super::*;

    #[test]
    fn test_decode() {
        let mut video_111 = Video::default();

        video_111
            .decode("tests/footages/1-1-1_2frames_prores422.mov")
            .expect("Some issue occur when decoding '1-1-1_2frames_prores422.mov'.");

        let expected_result_111 = Video {
            colr_atom: ColrAtom {
                size: 18,
                _color_parameter_type: Nclc,
                offset: 1234280,
                primary_index: 1,
                transfer_function_index: 1,
                matrix_index: 1,
                matched: true,
            },
            gama_atom: GamaAtom {
                size: 0,
                gama_value: 0,
                offsets: [].to_vec(),
                the_actual_gama_offset: 0,
                matched: false,
            },
            frames: [
                ProResFrame {
                    offset: 40,
                    frame_size: 616448,
                    _frame_id: 0.0,
                    frame_header_size: 148,
                    color_primaries: 1,
                    transfer_characteristic: 1,
                    matrix_coefficients: 1,
                },
                ProResFrame {
                    offset: 616488,
                    frame_size: 617195,
                    _frame_id: 0.0,
                    frame_header_size: 148,
                    color_primaries: 1,
                    transfer_characteristic: 1,
                    matrix_coefficients: 1,
                },
            ]
            .to_vec(),
            frame_count: 2,
        };

        let mut video_121 = Video::default();

        video_121
            .decode("tests/footages/1-2-1_2frames_prores422.mov")
            .expect("Some issue occur when decoding '1-2-1_2frames_prores422.mov'.");

        let expected_result_121 = Video {
            colr_atom: ColrAtom {
                size: 18,
                _color_parameter_type: Nclc,
                offset: 1234292,
                primary_index: 1,
                transfer_function_index: 2,
                matrix_index: 1,
                matched: true,
            },
            gama_atom: GamaAtom {
                size: 12,
                gama_value: 157286,
                offsets: [1234280].to_vec(),
                the_actual_gama_offset: 1234280,
                matched: true,
            },
            frames: [
                ProResFrame {
                    offset: 40,
                    frame_size: 616448,
                    _frame_id: 0.0,
                    frame_header_size: 148,
                    color_primaries: 1,
                    transfer_characteristic: 2,
                    matrix_coefficients: 1,
                },
                ProResFrame {
                    offset: 616488,
                    frame_size: 617195,
                    _frame_id: 0.0,
                    frame_header_size: 148,
                    color_primaries: 1,
                    transfer_characteristic: 2,
                    matrix_coefficients: 1,
                },
            ]
            .to_vec(),
            frame_count: 2,
        };

        assert_eq!(video_111, expected_result_111);
        assert_eq!(video_121, expected_result_121);
    }
}
