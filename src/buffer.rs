use anyhow::Result;
use std::io::{Error, ErrorKind};

/// buffer reader
///
/// example
/// ```
/// use protobuf_lite::buffer::Reader;
/// fn main() {
///     let data: Vec<u8> = vec![1,2,3,4,5,6,7,8];
///     let mut reader = Reader::new(data.as_slice());
/// }
/// ```
pub struct Reader<T> {
    buf: T,
    pos: usize,
    remain: usize,
}

impl<T> Reader<T>
where
    T: AsRef<[u8]>,
{
    /// create a buffer reader
    ///
    /// example
    /// ```
    /// use protobuf_lite::buffer::Reader;
    /// fn main() {
    ///     let data: Vec<u8> = vec![1,2,3,4,5,6,7,8];
    ///     let mut reader = Reader::new(data.as_slice());
    /// }
    /// ```
    pub fn new(data: T) -> Self {
        let length = data.as_ref().len();
        Reader {
            buf: data,
            pos: 0,
            remain: length,
        }
    }

    /// reset reader position
    #[inline]
    pub fn reset(&mut self) {
        self.pos = 0;
        self.remain = self.buf.as_ref().len();
    }

    /// gets remaining length of the buffer
    #[inline]
    pub fn remaining(&self) -> usize {
        self.remain
    }

    /// skip specified byte of data
    #[inline]
    pub fn skip(&mut self, n: usize) -> Result<()> {
        if self.remain < n {
            return Err(Error::new(ErrorKind::UnexpectedEof, "unexpected EOF").into());
        }
        self.pos += n;
        self.remain -= n;
        Ok(())
    }

    /// check if the buffer pointer is at the end
    #[inline]
    pub fn is_end(&self) -> bool {
        self.remain == 0
    }

    /// read 1 byte of data
    #[inline]
    pub fn read_byte(&mut self) -> Result<u8> {
        if self.remain < 1 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "unexpected EOF").into());
        }
        let b = self.buf.as_ref()[self.pos];
        self.pos += 1;
        self.remain -= 1;
        Ok(b)
    }

    /// read specified byte of data
    #[inline]
    pub fn read_bytes(&mut self, n: usize) -> Result<&[u8]> {
        if self.remain < n {
            return Err(Error::new(ErrorKind::UnexpectedEof, "unexpected EOF").into());
        }
        let b = &self.buf.as_ref()[self.pos..self.pos + n];
        self.pos += n;
        self.remain -= n;
        Ok(b)
    }

    /// read all remaining data
    #[inline]
    pub fn read_all_bytes(&mut self) -> Result<&[u8]> {
        self.read_bytes(self.remain)
    }

    /// read data to specified buffer, fills the buffer
    #[inline]
    pub fn read_bytes_into(&mut self, dst: &mut [u8]) -> Result<()> {
        dst.copy_from_slice(self.read_bytes(dst.len())?);
        Ok(())
    }
}
