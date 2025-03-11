use crate::buffer::Reader;
use crate::error::DecodeError;
use anyhow::Result;
use std::io::Write;

pub const MAX_VARINT_LENGTH: usize = 10;

#[inline]
pub fn write_uvarint<T>(mut x: u64, buf: &mut T) -> Result<()>
where
    T: Write,
{
    while x >= 0x80 {
        buf.write_all(&[x as u8 | 0x80])?;
        x >>= 7;
    }
    buf.write_all(&[x as u8])?;
    Ok(())
}

#[inline]
pub fn write_varint<T>(x: i64, buf: &mut T) -> Result<()>
where
    T: Write,
{
    let mut ux = (x as u64) << 1;
    if x < 0 {
        ux = !ux;
    }
    write_uvarint(ux, buf)?;
    Ok(())
}

#[inline]
pub fn encode_uvarint(x: u64) -> Vec<u8> {
    let mut buf = Vec::with_capacity(MAX_VARINT_LENGTH);
    let _ = write_uvarint(x, &mut buf);
    buf
}

#[inline]
pub fn encode_varint(x: i64) -> Vec<u8> {
    let mut buf = Vec::with_capacity(MAX_VARINT_LENGTH);
    let _ = write_varint(x, &mut buf);
    buf
}

#[inline]
pub fn read_uvarint<T>(buf: &mut Reader<T>) -> Result<u64>
where
    T: AsRef<[u8]>,
{
    let mut x: u64 = 0;
    let mut shift = 0;
    if buf.is_end() {
        return Err(DecodeError::EOF.into());
    }
    loop {
        match buf.read_byte() {
            Ok(v) => {
                let b = v as u64;
                x |= (b & 0x7F) << shift;
                shift += 7;
                if (b & 0x80) == 0 {
                    return Ok(x);
                }
                if shift >= 64 {
                    return Err(DecodeError::OverFlow64Bit.into());
                }
            }
            Err(_) => {
                return Err(DecodeError::UnexpectedEof.into());
            }
        }
    }
}

#[inline]
pub fn read_varint<T>(buf: &mut Reader<T>) -> Result<i64>
where
    T: AsRef<[u8]>,
{
    let ux = read_uvarint(buf)?;
    let mut x = (ux as i64) >> 1;
    if ux & 1 != 0 {
        x = !x;
    }
    Ok(x)
}
