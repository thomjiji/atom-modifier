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
fn create_backup_file(input_file_path: &Path) -> io::Result<()> {
    let original_stem = input_file_path
        .file_stem()
        .and_then(|os_str| os_str.to_str());

    let original_ext = input_file_path
        .extension()
        .and_then(|os_str| os_str.to_str());

    let mut backup_file_path;
    let mut new_filename;
    match (original_stem, original_ext) {
        (None, None) => {
            new_filename = String::from("Original");
            backup_file_path = input_file_path.with_file_name(&new_filename);
        }
        (None, Some(ext)) => {
            new_filename = format!("Original.{}", ext);
            backup_file_path = input_file_path.with_file_name(&new_filename);
        }
        (Some(stem), None) => {
            new_filename = format!("{}_Original", stem);
            backup_file_path = input_file_path.with_file_name(&new_filename);
        }
        (Some(stem), Some(ext)) => {
            new_filename = format!("{}_Original.{}", stem, ext);
            backup_file_path = input_file_path.with_file_name(&new_filename);
        }
    };

    let mut suffix = 1;
    while backup_file_path.exists() {
        new_filename = match original_ext {
            None => format!(
                "{}_Original_{}",
                original_stem.unwrap_or("Original"),
                suffix
            ),
            Some(ext) => format!(
                "{}_Original_{}.{}",
                original_stem.unwrap_or("Original"),
                suffix,
                ext
            ),
        };
        backup_file_path = input_file_path.with_file_name(&new_filename);
        suffix += 1;
    }

    // Copy the file
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
    create_backup_file(Path::new(&args.input_file_path))
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
