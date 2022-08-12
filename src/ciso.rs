use std::fmt::{self, Debug, Display};
use std::fs::File;
use std::io::{self, BufReader, Cursor, Read, Seek};
use std::mem::size_of;
use std::path::Path;
use std::str::Utf8Error;

use byteorder::{LittleEndian, ReadBytesExt};
use serde::Deserialize;

pub enum CisoError {
    DecompressError(&'static str),
    IoError(io::Error),
    BincodeError(bincode::ErrorKind),
    Utf8Error(Utf8Error),
}

impl From<io::Error> for CisoError {
    fn from(e: io::Error) -> Self {
        CisoError::IoError(e)
    }
}

impl From<Box<bincode::ErrorKind>> for CisoError {
    fn from(e: Box<bincode::ErrorKind>) -> Self {
        CisoError::BincodeError(*e)
    }
}

impl From<Utf8Error> for CisoError {
    fn from(e: Utf8Error) -> Self {
        CisoError::Utf8Error(e)
    }
}

impl Debug for CisoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&format!("{:?}", &self), f)
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

    let header: Header = bincode::deserialize_from(Cursor::new(buf))?;
    println!("Header: {:#?}", header);

    Ok(header)
}

pub fn decompress_ciso(ciso_file: &Path, _output_file: &Path) -> Result<()> {
    let f = File::open(ciso_file)?;

    let mut reader = BufReader::new(f);
    let header = read_header(&mut reader)?;

    let magic = std::str::from_utf8(&header.magic)?;
    if magic != "CISO" || header.block_size == 0 || header.total_bytes == 0 {
        return Err(CisoError::DecompressError("Invalid header"));
    }

    if header.align != 0 {
        return Err(CisoError::DecompressError("Align != 0 not supported"));
    }

    let total_blocks = header.total_bytes as usize / header.block_size as usize;

    let mut index_buffer = vec![0u32; total_blocks + 1];
    reader.read_u32_into::<LittleEndian>(&mut index_buffer)?;

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
        };
    }

    Ok(())
}
