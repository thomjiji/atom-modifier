use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

fn main() -> std::io::Result<()> {
    find_and_write_atom(
        "/Users/thom/Desktop/video_stream_oneframe_modified.mov",
        "gama",
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

    if let Some(position) = find_atom_position(&mut file, atom.as_bytes())? {
        // write_bytes_at(&mut file, position + 11, &[0x01, 0x00, 0x01, 0x00, 0x01])?;
        write_bytes_at(&mut file, position + 6, &[0x33, 0x33, 0x00, 0x00])?;
    } else {
        eprintln!("Did not find the string to modify.");
    }

    println!("{:?}", atom.as_bytes());

    Ok(())
}

/// Find provided atom name's position, returned by decimal.
fn find_atom_position(f: &mut std::fs::File, s: &[u8]) -> std::io::Result<Option<u64>> {
    const BUFFER_SIZE: usize = 1024;
    let mut buffer = [0; BUFFER_SIZE];

    // The position in `s` that we're looking for in the buffer.
    let mut offset = 0;

    loop {
        match f.read(&mut buffer)? {
            0 => return Ok(None), // End of file, string not found
            len => {
                for (i, &byte) in buffer[..len].iter().enumerate() {
                    if byte == s[offset] {
                        offset += 1;
                        if offset == s.len() {
                            // Found the string, calculate position
                            let buffer_pos = f.stream_position()? as usize;
                            let pos = buffer_pos - BUFFER_SIZE + i + 1 - offset;
                            return Ok(Some(pos as u64));
                        }
                    } else if offset > 0 {
                        // If there was a partial match, we need to get back those bytes
                        // and should step file cursor back.
                        f.seek(SeekFrom::Current(-(offset as i64 - 1)))?;
                        offset = 0;
                    }
                }
            }
        }
    }
}

fn write_bytes_at(f: &mut std::fs::File, position: u64, bytes: &[u8]) -> std::io::Result<()> {
    f.seek(SeekFrom::Start(position))?;
    f.write_all(bytes)
}