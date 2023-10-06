use clap::Parser;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Instant;

use atom_modifier::Args;
use atom_modifier::Video;

fn main() {
    let args = Args::parse();

    let now = Instant::now();
    let mut video = Video::default();
    video
        .decode(args.input_file_path.as_str())
        .unwrap_or_else(|e| {
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
    video
        .encode(
            &mut file,
            &video,
            args.primary_index,
            args.transfer_function_index,
            args.matrix_index,
            args.gama_value,
        )
        .expect("Encode has some problem.");
    println!(
        "- Time elapsed after encoding the file: {:?}",
        now.elapsed()
    );

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("output.log")
        .expect("Failed to write/find the log file.");
    writeln!(file, "{:#?}", video).unwrap();
    // println!("{:#?}", video);
}
