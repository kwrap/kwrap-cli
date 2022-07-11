use std::io::{Error, ErrorKind, Read, Result};

const ID: [u8; 6] = *b"\xffKWRAP";

const VERSION: u8 = 1;

// DATA: JSON '[]' 2
// NONCE: 12, DATA: N, TAG: 16
const MINIMUM_DATA: usize = 12 + 2 + 16;

#[derive(Debug)]
pub struct KwrapFile {
    // id: [u8; 6],
    // version: u8,
    pub salt: [u8; 32],
    pub iterations: u32,
    pub data: Vec<u8>,
}

impl KwrapFile {
    pub fn parse<R: Read>(mut r: R) -> Result<Self> {
        Self::read_id(&mut r)?;
        Self::read_vsersion(&mut r)?;
        let salt = Self::read_salt(&mut r)?;
        let iterations = Self::read_iterations(&mut r)?;
        let data = Self::read_data(&mut r)?;
        Ok(Self {
            // id: ID,
            // version: VERSION,
            salt,
            iterations,
            data,
        })
    }

    fn read_id<R: Read>(r: &mut R) -> Result<()> {
        let mut buf = [0; 6];
        r.read_exact(&mut buf)?;
        if buf == ID {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::Other, "Kwrap file ID Error"))
        }
    }

    fn read_vsersion<R: Read>(r: &mut R) -> Result<()> {
        let mut buf = [0; 1];
        r.read_exact(&mut buf)?;
        if buf == [VERSION] {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::Other, "Kwrap version Error"))
        }
    }

    fn read_salt<R: Read>(r: &mut R) -> Result<[u8; 32]> {
        let mut buf = [0; 32];
        r.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_iterations<R: Read>(r: &mut R) -> Result<u32> {
        let mut buf = [0; 4];
        r.read_exact(&mut buf)?;
        let iterations = u32::from_be_bytes(buf);
        assert_ne!(iterations, 0);
        Ok(iterations)
    }

    fn read_data<R: Read>(r: &mut R) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        r.read_to_end(&mut buf)?;
        assert!(buf.len() >= MINIMUM_DATA);
        Ok(buf)
    }

    // pub fn to_vec(&self) -> Vec<u8> {
    //     let mut bytes = Vec::with_capacity(6 + 1 + 32 + 4 + self.data.len());
    //     bytes.extend_from_slice(&self.id);
    //     bytes.extend_from_slice(&[self.version]);
    //     bytes.extend_from_slice(&self.salt);
    //     bytes.extend_from_slice(&self.iterations.to_be_bytes());
    //     bytes.extend_from_slice(&self.data);
    //     return bytes;
    // }
}
