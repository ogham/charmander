use std::io;
use std::io::Read;
use std::str::from_utf8;

use rustc_unicode::str::utf8_char_width;


pub struct Chars<R> {
    pub inner: R,
}

impl<R: Read> Chars<R> {
    pub fn new(r: R) -> Chars<R> {
        Chars { inner: r }
    }
}

pub enum ReadBytes {
    FirstByte(u8),
    WholeBuffer([u8; 4], usize)
}

pub enum ReadChar {
    Ok(char, ReadBytes),
    Invalid(ReadBytes),
}

impl<R: Read> Iterator for Chars<R> {
    type Item = Result<ReadChar, io::Error>;

    fn next(&mut self) -> Option<Result<ReadChar, io::Error>> {
        let mut buf = [0];
        let first_byte = match self.inner.read(&mut buf) {
            Ok(0)   => return None,
            Ok(_)   => buf[0],
            Err(e)  => return Some(Err(e)),
        };

        let read = ReadBytes::FirstByte(first_byte);
        let width = match utf8_char_width(first_byte) {
            0 => return Some(Ok(ReadChar::Invalid(read))),
            1 => return Some(Ok(ReadChar::Ok(first_byte as char, read))),
            w => w,
        };

        assert! { width <= 4 };

        let mut buf = [first_byte, 0, 0, 0];
        let mut start = 1;

        while start < width {
            match self.inner.read(&mut buf[start..width]) {
                Ok(0)   => return Some(Ok(ReadChar::Invalid(ReadBytes::WholeBuffer(buf, width)))),
                Ok(n)   => start += n,
                Err(e)  => return Some(Err(e)),
            }
        }

        let read = ReadBytes::WholeBuffer(buf, width);
        match from_utf8(&buf[..width]) {
            Ok(s)  => Some(Ok(ReadChar::Ok(s.char_at(0), read))),
            Err(_) => Some(Ok(ReadChar::Invalid(read))),
        }
    }
}