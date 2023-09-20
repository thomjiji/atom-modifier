use std::fs::OpenOptions;
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};

fn main() -> io::Result<()> {
    find_hdlr()
}

fn find_hdlr() -> io::Result<()> {
    // Open file with write permissions
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/Users/thom/Desktop/video_stream_oneframe_modified.mov")?;

    // Seek to the position
    let position_to_modify = 0x8fe30 + 11;
    // The position is adjusted counting bytes beginning from offset 0x8fe30.
    f.seek(SeekFrom::Start(position_to_modify as u64))?;

    // Write new data
    f.write_all(&[0x01, 0x00, 0x01, 0x00, 0x01])?;

    Ok(())
}