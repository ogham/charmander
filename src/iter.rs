//! Custom iterator for reading UTF-8 characters from strings.
//!
//! Our iterator differs from the `std::io::Chars` iterator as it is allowed
//! to return *invalid* UTF-8 characters, whereas `Chars` can only have the
//! entire string succeed or entirely fail. The only way this iterator can
//! fail is if there's an IO error. A normal program would be correct to throw
//! an error if an input string isn't valid UTF-8, but charmander should
//! definitely not be crashing from this!

use std::io::Read;
use std::io::Error as IOError;
use std::str::from_utf8;

use rustc_unicode::str::utf8_char_width;


/// Iterator over the UTF-8 characters in a string.
pub struct Chars<R> {
    inner: R,
}

impl<R: Read> Chars<R> {

    /// Create a new `Chars` iterator, based on the given inner iterator.
    pub fn new(r: R) -> Chars<R> {
        Chars { inner: r }
    }
}

/// The byte buffer that's used when reading in characters.
pub enum ReadBytes {

    /// Only one byte was necessary to determine success or failure.
    FirstByte(u8),

    /// More than one byte was necessary: this holds a four-byte buffer along
    /// with the number of bytes actually taken up by the character.
    WholeBuffer([u8; 4], usize)
}

/// A read from the stream without any IO errors.
pub enum ReadChar {

    /// The character was valid UTF-8, so the character and the byte buffer
    /// get returned.
    Ok(char, ReadBytes),

    /// The character was **not** valid UTF-8, so there's no `char` to return!
    /// Just the buffer gets returned.
    Invalid(ReadBytes),
}

impl<R: Read> Iterator for Chars<R> {
    type Item = Result<ReadChar, IOError>;

    fn next(&mut self) -> Option<Result<ReadChar, IOError>> {

        // Read the first byte from the stream into a one-byte buffer.
        let mut buf = [0];
        let first_byte = match self.inner.read(&mut buf) {
            Ok(0)   => return None,
            Ok(_)   => buf[0],
            Err(e)  => return Some(Err(e)),
        };

        // Examine the byte to test:
        // - whether it's a continuation byte as the first byte (an error);
        // - whether it's a one-byte character and needs no further processing.
        let read = ReadBytes::FirstByte(first_byte);
        let width = match utf8_char_width(first_byte) {
            0 => return Some(Ok(ReadChar::Invalid(read))),
            1 => return Some(Ok(ReadChar::Ok(first_byte as char, read))),
            w => w,
        };

        // There are no characters above four bytes, so anything above this
        // is an error!
        assert! { width <= 4 };

        // Read in the rest of the bytes.
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