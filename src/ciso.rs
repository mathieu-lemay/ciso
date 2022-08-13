use std::fmt::Debug;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Cursor, Read, Write};
use std::mem::size_of;
use std::path::Path;
use std::str::Utf8Error;

use byteorder::{LittleEndian, ReadBytesExt};
use flate2::bufread::DeflateDecoder;
use serde::Deserialize;

#[derive(Debug)]
pub enum CisoError {
    Decompress(&'static str),
    Io(io::Error),
    Bincode(bincode::ErrorKind),
    Utf8(Utf8Error),
}

impl From<io::Error> for CisoError {
    fn from(e: io::Error) -> Self {
        CisoError::Io(e)
    }
}

impl From<Box<bincode::ErrorKind>> for CisoError {
    fn from(e: Box<bincode::ErrorKind>) -> Self {
        CisoError::Bincode(*e)
    }
}

impl From<Utf8Error> for CisoError {
    fn from(e: Utf8Error) -> Self {
        CisoError::Utf8(e)
    }
}

type Result<T> = std::result::Result<T, CisoError>;

#[derive(Debug, Deserialize)]
struct Header {
    magic: [u8; 4],
    _header_size: u32,
    total_bytes: u64,
    block_size: u32,
    _version: u8,
    align: u8,
    _reserved: [u8; 2],
}

fn read_header<T: Read>(reader: &mut T) -> Result<Header> {
    let mut buf = [0; size_of::<Header>()];

    reader.read_exact(&mut buf)?;

    Ok(bincode::deserialize_from(Cursor::new(buf))?)
}

pub fn decompress_ciso(ciso_file: &Path, output_file: &Path) -> Result<()> {
    let in_f = File::open(ciso_file)?;

    let mut reader = BufReader::new(in_f);
    let header = read_header(&mut reader)?;

    let magic = std::str::from_utf8(&header.magic)?;
    if magic != "CISO" || header.block_size == 0 || header.total_bytes == 0 {
        return Err(CisoError::Decompress("Invalid header"));
    }

    if header.align != 0 {
        return Err(CisoError::Decompress("Align != 0 not supported"));
    }

    let total_blocks = header.total_bytes as usize / header.block_size as usize;

    let mut index_buffer = vec![0u32; total_blocks + 1];
    reader.read_u32_into::<LittleEndian>(&mut index_buffer)?;

    let mut output_buffer = vec![0u8; header.block_size as usize];

    let out_f = File::create(output_file)?;
    let mut writer = BufWriter::new(out_f);

    let mut input_buffer = Vec::new();

    for block in 0..total_blocks {
        let index = index_buffer[block] & 0x7fffffff;
        let plain = (index_buffer[block] & 0x80000000) > 0;

        let read_size = if plain {
            header.block_size
        } else {
            let index2 = index_buffer[block + 1] & 0x7fffffff;
            if index2 < index {
                println!(
                    "block: {}, total: {}, index: {}, index2: {}",
                    block, total_blocks, index, index2
                )
            }
            index2 - index
        } as u64;

        input_buffer.resize(read_size as usize, 0);
        reader.read_exact(&mut input_buffer)?;

        if plain {
            writer.write_all(&input_buffer)?;
        } else {
            let mut z = DeflateDecoder::new(&input_buffer[..]);
            let out_size = z.read_to_end(&mut output_buffer)?;
            writer.write_all(&output_buffer[0..out_size])?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn test_something() {
        assert!(false);
    }
}
