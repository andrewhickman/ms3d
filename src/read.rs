use std::io;

pub(crate) trait BufReadExact {
    fn buf_read_exact(&mut self, len: usize) -> io::Result<&[u8]>;
}

pub(crate) struct IoReader<R: io::Read> {
    rdr: R,
    buf: Vec<u8>,
}

impl<R: io::Read> IoReader<R> {
    pub fn new(rdr: R) -> Self {
        IoReader { rdr, buf: Vec::new() }
    }
}

impl<R: io::Read> BufReadExact for IoReader<R> {
    fn buf_read_exact(&mut self, len: usize) -> io::Result<&[u8]> {
        self.buf.resize(len, 0);
        self.rdr.read_exact(self.buf.as_mut_slice())?;
        Ok(self.buf.as_slice())
    }
}

pub(crate) struct SliceReader<'a> {
    slice: &'a [u8],
}

impl<'a> SliceReader<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        SliceReader { slice }
    }
}

impl<'a> BufReadExact for SliceReader<'a> {
    fn buf_read_exact(&mut self, len: usize) -> io::Result<&[u8]> {
        if len > self.slice.len() {
            return Err(io::ErrorKind::UnexpectedEof.into())
        }
        let (head, tail) = self.slice.split_at(len);
        self.slice = tail;
        Ok(head)
    }
}