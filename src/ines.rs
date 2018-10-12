// https://wiki.nesdev.com/w/index.php/INES

use std::fs::File;
use std::path::Path;
use std::io::Read;
use error::{Error, ResultContext};

#[derive(Debug)]
pub struct Header {
    n_prgrom: u8, // size of PRG ROM in 16 KB units
    n_chrrom: u8, // size of CHR ROM in 8 KB units
    flag6: u8,
    flag7: u8,
    n_prgram: u8, // size of PRG RAM in 8 KB units
    flag9: u8,
    flag10: u8,
}

#[derive(Debug)]
pub struct Ines {
    header: Header,
    prgrom: Vec<u8>,
    chrrom: Vec<u8>,
}

impl Ines {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut file = File::open(path)?;

        let mut bytes = [0u8;11];
        file.read_exact(&mut bytes)?;
        if &bytes[0..4] != &[b'N', b'E', b'S', 0x1a] {
            return Err(Error::new("not a NES file".to_string()));
        }
        let header = Header {
            n_prgrom: bytes[4],
            n_chrrom: bytes[5],
            flag6:    bytes[6],
            flag7:    bytes[7],
            n_prgram: bytes[8],
            flag9:    bytes[9],
            flag10:   bytes[10],
        };
        let mut prgrom = Vec::new();
        let mut chrrom = Vec::new();
        prgrom.resize(header.n_prgrom as usize * 16 * 1024, 0);
        chrrom.resize(header.n_chrrom as usize * 8 * 1024, 0);
        file.read_exact(&mut prgrom)?;
        file.read_exact(&mut chrrom)?;

        Ok(Ines { header, prgrom, chrrom })
    }
}
