use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::{path::Path, time::Instant};

use clap::Parser;

use atom_modifier::Args;
use atom_modifier::Video;

/// Creates a backup file for the given input file path. The backup file name will be in
/// the format "{filename}_Original.{ext}".
///
/// If a file with that name already exists, the function will append a suffix to the
/// filename until it finds a unique name.
///
/// # Arguments
///
/// * `input_file_path` - A reference to the path of the input file.
///
/// # Returns
///
/// A `Result` containing `()` if the operation succeeds, or an `io::Error` if the
/// operation fails.
fn backup_input_file(input_file_path: &Path) -> io::Result<()> {
    // Extract the stem (filename without extension) from the input file path
    let original_stem = input_file_path
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("Original");

    // Extract the extension from the input file path
    let original_ext = input_file_path
        .extension()
        .and_then(OsStr::to_str);

    // Create the initial backup file name
    let mut new_filename = match original_ext {
        Some(ext) => format!("{}_Original.{}", original_stem, ext),
        None => format!("{}_Original", original_stem),
    };

    // Create the initial backup file path
    let mut backup_file_path = input_file_path.with_file_name(&new_filename);

    // If a file with the backup file name already exists, append a suffix to the filename
    let mut suffix = 1;
    while backup_file_path.exists() {
        new_filename = match original_ext {
            Some(ext) => format!("{}_Original_{}.{}", original_stem, suffix, ext),
            None => format!("{}_Original_{}", original_stem, suffix),
        };
        backup_file_path = input_file_path.with_file_name(&new_filename);
        suffix += 1;
    }

    // Copy the file to the backup file path
    std::fs::copy(input_file_path, &backup_file_path)?;

    Ok(())
}

fn main() {
    let args = Args::parse();

    // Decoding
    let now = Instant::now();
    let mut video = Video::default();
    video
        .decode(args.input_file_path.as_str())
        .unwrap_or_else(|e| {
            eprintln!(
                "Error decoding input file '{}': {}",
                args.input_file_path, e
            );
            std::process::exit(1);
        });
    println!(
        "- Time elapsed after decoding the file: {:?}",
        now.elapsed()
    );

    // Make a backup of the original file name as "<filename>_Original.<ext>".
    backup_input_file(Path::new(&args.input_file_path))
        .expect("encountered an error while creating a backup of input file");

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&args.input_file_path)
        .unwrap_or_else(|e| {
            eprintln!(
                "Error trying to open file '{}' in reading/writing mode: {}",
                args.input_file_path, e
            );
            std::process::exit(1);
        });

    // Encoding
    let now = Instant::now();
    video
        .encode(
            &mut file,
            &video,
            args.primary_index,
            args.transfer_function_index,
            args.matrix_index,
            args.gama_value,
        )
        .unwrap_or_else(|e| {
            eprintln!("Error encoding the file '{}': {}", args.input_file_path, e);
            std::process::exit(1);
        });
    println!(
        "- Time elapsed after encoding the file: {:?}",
        now.elapsed()
    );

    // Logging
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("output.log")
        .expect("Failed to write/find the log file.");
    writeln!(file, "{:#?}", video).unwrap();
    // println!("{:#?}", video);
}
