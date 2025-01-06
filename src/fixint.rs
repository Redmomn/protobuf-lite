use crate::buffer::Reader;
use anyhow::Result;

#[inline]
pub fn write_fix32(x: i32, buf: &mut Vec<u8>) {
    buf.extend_from_slice(x.to_le_bytes().as_slice());
}

#[inline]
pub fn write_fix64(x: i64, buf: &mut Vec<u8>) {
    buf.extend_from_slice(x.to_le_bytes().as_slice());
}

#[inline]
pub fn encode_fix32(x: i32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(size_of::<i32>());
    buf.extend_from_slice(&x.to_le_bytes());
    buf
}

#[inline]
pub fn encode_fix64(x: i64) -> Vec<u8> {
    let mut buf = Vec::with_capacity(size_of::<i64>());
    buf.extend_from_slice(&x.to_le_bytes());
    buf
}

#[inline]
pub fn read_fix32(buf: &mut Reader) -> Result<i32> {
    let mut b = [0u8; size_of::<i32>()];
    buf.read_bytes_into(&mut b)?;
    Ok(i32::from_le_bytes(b))
}

#[inline]
pub fn read_fix64(buf: &mut Reader) -> Result<i64> {
    let mut b = [0u8; size_of::<i64>()];
    buf.read_bytes_into(&mut b)?;
    Ok(i64::from_le_bytes(b))
}
