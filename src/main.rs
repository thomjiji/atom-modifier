use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

fn main() -> std::io::Result<()> {
    find_and_write_atom(
        "/Users/thom/Desktop/video_stream_oneframe_modified.mov",
        "colr",
    )
    .map_err(|e| {
        eprintln!("Failed to find and write nclc: {}", e);
        e
    })
}

fn find_and_write_atom(input_file_path: &str, atom: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(input_file_path)?;

    if let Some(position) = find_string_position(&mut file, atom.as_bytes())? {
        write_bytes_at(&mut file, position + 11, &[0x01, 0x00, 0x01, 0x00, 0x01])?;
    } else {
        eprintln!("Did not find the string to modify.");
    }

    Ok(())
}

fn find_string_position(f: &mut std::fs::File, s: &[u8]) -> std::io::Result<Option<u64>> {
    let mut buffer = [0; 1]; // Buffer to hold one byte
    let mut offset = 0;

    while f.read(&mut buffer)? > 0 {
        if buffer[0] == s[offset] {
            offset += 1;
            if offset == s.len() {
                return Ok(Some(f.stream_position()? - s.len() as u64)); // Here the pattern ends
            }
        } else {
            offset = 0;
        }
    }
    Ok(None) // Could not find the string
}

fn write_bytes_at(f: &mut std::fs::File, position: u64, bytes: &[u8]) -> std::io::Result<()> {
    f.seek(SeekFrom::Start(position))?;
    f.write_all(bytes)
}