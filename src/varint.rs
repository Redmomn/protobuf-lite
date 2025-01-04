use crate::error::DecodeError;
use anyhow::Result;
use std::io::Read;

pub const MAX_VARINT_LENGTH: usize = 10;

#[inline]
pub fn write_uvarint(mut x: u64, buf: &mut Vec<u8>) {
    while x >= 0x80 {
        buf.push(x as u8 | 0x80);
        x >>= 7;
    }
    buf.push(x as u8);
}

#[inline]
pub fn write_varint(x: i64, buf: &mut Vec<u8>) {
    let mut ux = (x as u64) << 1;
    if x < 0 {
        ux = !ux;
    }
    write_uvarint(ux, buf);
}

#[inline]
pub fn encode_uvarint(x: u64) -> Vec<u8> {
    let mut buf = Vec::with_capacity(MAX_VARINT_LENGTH);
    write_uvarint(x, &mut buf);
    buf
}

#[inline]
pub fn encode_varint(x: i64) -> Vec<u8> {
    let mut buf = Vec::with_capacity(MAX_VARINT_LENGTH);
    write_varint(x, &mut buf);
    buf
}

#[inline]
pub fn read_uvarint(buf: &mut impl Read) -> Result<u64> {
    let mut x: u64 = 0;
    let mut b = [0u8; 1];
    let mut shift = 0;
    loop {
        buf.read_exact(&mut b)?;
        let b = b[0] as u64;
        x |= (b & 0x7F) << shift;
        shift += 7;
        if (b & 0x80) == 0 {
            return Ok(x);
        }
        if shift >= 64 {
            return Err(DecodeError::OverFlow64Bit.into());
        }
    }
}

#[inline]
pub fn read_varint(buf: &mut impl Read) -> Result<i64> {
    let ux = read_uvarint(buf)?;
    let mut x = (ux as i64) >> 1;
    if ux & 1 != 0 {
        x = !x;
    }
    Ok(x)
}
