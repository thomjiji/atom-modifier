use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::{env, process};

struct Video {
    file: File,
    size: u64,
    offset: u64,
    cursor: u64,
}

// impl Video {
//     fn new(input_file: &File) {
//         Video {
//             file: input_file,
//             size: input_file.metadata()?.len(),
//             offset: 0,
//             cursor: 0,
//         };
//     }
// }

#[derive(Debug)]
struct Config {
    file_path: File,
    colr: String,
    gama: String,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file path"),
        };

        let file_path = File::open(file_path).expect("Failed to open file");

        let colr = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a colr tag"),
        };

        let gama = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get gama tag"),
        };

        Ok(Config {
            file_path,
            colr,
            gama,
        })
    }
}

/// write_bytes_at(&mut file, position + 6, &[0x33, 0x33, 0x00, 0x00])?;  // change gama to 2.2
/// write_bytes_at(&mut file, position + 6, &[0x66, 0x66, 0x00, 0x00])?;  // change gama to 2.4
/// write_bytes_at(&mut file, position, &[0x00, 0x00, 0x00, 0x00])?;      // remove gama atom
fn find_and_write_atom(input_file_path: &str, atom: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(input_file_path)?;

    if let Some(position) = find_atom_position(&mut file, atom.as_bytes())? {
        write_bytes_at(&mut file, position + 12, &[0x01, 0x00, 0x01, 0x00, 0x01])?; // change from 1-2-1 to 1-1-1
        println!("{position}");
    } else {
        eprintln!("Did not find the string to modify.");
    }

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

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    println!("{:#?}", config);

    // find_and_write_atom(
    //     "/Users/thom/code/rust/atom_modifier/test_footages/1-2-1_modified.mov",
    //     "colr",
    // )
    // .map_err(|e| {
    //     eprintln!("Failed to find and write nclc: {}", e);
    //     e
    // })
}