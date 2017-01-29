use byteorder::{BigEndian, ByteOrder};
use std::io;

#[inline]
fn check(buf: &[u8], len: usize) -> Result<(), io::Error> {
    if len > buf.len() {
        return Err(
            io::Error::new(io::ErrorKind::InvalidInput, "Input buffer size is not enough"))
    }
    Ok(())
}

#[inline]
pub fn read_u16(buf: &[u8]) -> Result<u16, io::Error> {
    check(buf, 2)?;
    Ok(BigEndian::read_u16(buf))
}

#[inline]
pub fn read_u32(buf: &[u8]) -> Result<u32, io::Error> {
    check(buf, 4)?;
    Ok(BigEndian::read_u32(buf))
}

#[inline]
pub fn read_u64(buf: &[u8]) -> Result<u64, io::Error> {
    check(buf, 8)?;
    Ok(BigEndian::read_u64(buf))
}

