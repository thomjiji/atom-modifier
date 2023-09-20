use std::fs::File;
use std::io;
use std::io::{Read, Seek, SeekFrom};

fn main() -> io::Result<()> {
    let mut f = File::open("/Users/thom/Desktop/video_stream_oneframe.mov")?;
    let mut buffer = [0; 10];

    f.seek(SeekFrom::Start(10)).expect("Something went wrong");
    f.read_exact(&mut buffer)?;

    // Print out bytes in hexadecimal
    let hexadecimal = buffer
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join(" ");

    let s: String = buffer.iter().map(|&b| b as char).collect();

    println!("{}", s);
    println!("{}", hexadecimal);

    Ok(())
}