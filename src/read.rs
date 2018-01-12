use std::io;

pub(crate) trait Read {
    fn read(&mut self, len: usize) -> io::Result<&[u8]>;
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

impl<R: io::Read> Read for IoReader<R> {
    fn read(&mut self, len: usize) -> io::Result<&[u8]> {
        unsafe {
            self.buf.reserve(len);
            let slice = self.buf.get_unchecked_mut(..len);
            self.rdr.read_exact(slice)?;
            Ok(slice)
        }
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

impl<'a> Read for SliceReader<'a> {
    fn read(&mut self, len: usize) -> io::Result<&[u8]> {
        if len > self.slice.len() {
            return Err(io::ErrorKind::UnexpectedEof.into())
        }
        let (head, tail) = self.slice.split_at(len);
        self.slice = tail;
        Ok(head)
    }
}