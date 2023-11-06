use std::ops::RangeInclusive;

use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(
    name = "atom_modifier",
    author = "thomjiji <noadstrackers@duck.com>",
    version = "0.0.1"
)]
#[command(
    about = "Modify color primaries, transfer characteristics, matrix coefficients, and gamma value of QuickTime file.",
    long_about = "This program allows you to modify the color primaries, transfer characteristics, matrix coefficients, and gamma value of QuickTime file. Before do the modification, it will create a backup of the input file."
)]
#[command(next_line_help = true)]
pub struct Args {
    #[arg(short, long = "input-file-path", value_name = "FILE", required = true)]
    /// The path to the input file
    pub input_file_path: String,

    #[arg(short, long = "color-primaries", value_name = "INDEX_VALUE", required = true, value_parser = color_primaries_value_check)]
    /// Change the "color primaries index" to <INDEX_VALUE>
    pub primary_index: u8,

    #[arg(
        short,
        long = "transfer-characteristics",
        value_name = "INDEX_VALUE",
        required = true,
        value_parser = transfer_characteristics_value_check,
    )]
    /// Change the "transfer characteristics index" to <INDEX_VALUE>
    pub transfer_function_index: u8,

    #[arg(
        short,
        long = "matrix-coefficients",
        value_name = "INDEX_VALUE",
        required = true,
        value_parser = matrix_coefficients_value_check,
    )]
    /// Change the "matrix coefficients index" to <INDEX_VALUE>
    pub matrix_index: u8,

    #[arg(short, long = "gama-value", default_value_t = -1.0, required = false)]
    /// The gamma value to set. If not present, defaults to -1.0
    pub gama_value: f32,

    #[arg(long = "modify-in-place", default_value_t = false, required = false)]
    /// If passed, modify the input file in-place. Otherwise, create a backup of input file. Defaults to false (create backup).
    pub modify_in_place: bool,
}

const PRIMARIES_RANGE: RangeInclusive<usize> = 0..=12;
const TRANSFER_FUNCTION_RANGE: RangeInclusive<usize> = 0..=18;
const MATRIX_RANGE: RangeInclusive<usize> = 0..=14;

const COLOR_PRIMARY_NAMES: [&str; 13] = [
    "Reserved",
    "ITU-R BT.709",
    "Unspecified",
    "Reserved",
    "ITU-R BT.470M",
    "ITU-R BT.470BG",
    "SMPTE 170M",
    "SMPTE 240M",
    "FILM",
    "ITU-R BT.2020",
    "SMPTE ST 428-1",
    "DCI P3",
    "P3 D65",
];

// Define constants
const TRANSFER_FUNCTION_NAMES: [&str; 19] = [
    "Reserved",
    "ITU-R BT.709",
    "Unspecified",
    "Reserved",
    "Gamma 2.2 curve",
    "Gamma 2.8 curve",
    "SMPTE 170M",
    "SMPTE 240M",
    "Linear",
    "Log",
    "Log Sqrt",
    "IEC 61966-2-4",
    "ITU-R BT.1361 Extended Colour Gamut",
    "IEC 61966-2-1",
    "ITU-R BT.2020 10 bit",
    "ITU-R BT.2020 12 bit",
    "SMPTE ST 2084 (PQ)",
    "SMPTE ST 428-1",
    "ARIB STD-B67 (HLG)",
];

const MATRIX_NAMES: [&str; 15] = [
    "GBR",
    "BT709",
    "Unspecified",
    "Reserved",
    "FCC",
    "BT470BG",
    "SMPTE 170M",
    "SMPTE 240M",
    "YCOCG",
    "BT2020 Non-constant Luminance",
    "BT2020 Constant Luminance",
    "SMPTE ST 2085 (2015)",
    "Chromaticity-derived non-constant luminance system",
    "Chromaticity-derived constant luminance system",
    "Rec. ITU-R BT.2100-0 ICTCP",
];

// Function to retrieve the name by index
fn get_color_primary_name(index: u8) -> Option<&'static str> {
    COLOR_PRIMARY_NAMES.get(index as usize).copied()
}

// Function to retrieve the name by index
fn get_transfer_function_name(index: u8) -> Option<&'static str> {
    TRANSFER_FUNCTION_NAMES.get(index as usize).copied()
}
// Function to retrieve the name by index
fn get_matrix_name(index: u8) -> Option<&'static str> {
    MATRIX_NAMES.get(index as usize).copied()
}

fn color_primaries_value_check(s: &str) -> Result<u8, String> {
    value_check(s, PRIMARIES_RANGE, "color primary", get_color_primary_name)
}

fn transfer_characteristics_value_check(s: &str) -> Result<u8, String> {
    value_check(
        s,
        TRANSFER_FUNCTION_RANGE,
        "transfer function",
        get_transfer_function_name,
    )
}

fn matrix_coefficients_value_check(s: &str) -> Result<u8, String> {
    value_check(s, MATRIX_RANGE, "matrix", get_matrix_name)
}

fn value_check<T>(
    s: &str,
    range: RangeInclusive<usize>,
    error_message: &str,
    from_index_fn: fn(u8) -> Option<T>,
) -> Result<u8, String>
where
    T: std::fmt::Display,
{
    let value: usize = s
        .parse()
        .map_err(|_| format!("`{}` isn't a {} index number", s, error_message))?;

    if range.contains(&value) {
        Ok(value as u8)
    } else {
        let valid_values = range
            .map(|index| format!("\t{} - {}\n", index, from_index_fn(index as u8).unwrap()))
            .collect::<Vec<_>>()
            .join("");
        Err(format!("\nValid values are: \n{}", valid_values))
    }
}
