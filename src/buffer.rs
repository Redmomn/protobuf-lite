use anyhow::Result;
use std::io::{Error, ErrorKind};

pub struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
    remain: usize,
}

impl Reader<'_> {
    pub fn new(data: &[u8]) -> Reader {
        Reader {
            buf: data,
            pos: 0,
            remain: data.len(),
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.pos = 0;
        self.remain = self.buf.len();
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.remain
    }

    #[inline]
    pub fn skip(&mut self, n: usize) -> Result<()> {
        if self.remain < n {
            return Err(Error::new(ErrorKind::UnexpectedEof, "unexpected EOF").into());
        }
        self.pos += n;
        self.remain -= n;
        Ok(())
    }

    #[inline]
    pub fn is_end(&self) -> bool {
        self.remain == 0
    }

    #[inline]
    pub fn read_byte(&mut self) -> Result<u8> {
        if self.remain < 1 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "unexpected EOF").into());
        }
        let b = self.buf[self.pos];
        self.pos += 1;
        self.remain -= 1;
        Ok(b)
    }

    #[inline]
    pub fn read_bytes(&mut self, n: usize) -> Result<&[u8]> {
        if self.remain < n {
            return Err(Error::new(ErrorKind::UnexpectedEof, "unexpected EOF").into());
        }
        let b = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        self.remain -= n;
        Ok(b)
    }

    #[inline]
    pub fn read_all_bytes(&mut self) -> Result<&[u8]> {
        self.read_bytes(self.remain)
    }

    #[inline]
    pub fn read_bytes_into(&mut self, dst: &mut [u8]) -> Result<()> {
        dst.copy_from_slice(self.read_bytes(dst.len())?);
        Ok(())
    }
}
