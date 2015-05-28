#![feature(exit_status, str_char, unicode)]

extern crate getopts;
extern crate ansi_term;
use ansi_term::Colour::*;
use ansi_term::Style::{self, Plain};

extern crate unicode_names;

extern crate rustc_unicode;
use rustc_unicode::str::utf8_char_width;

extern crate unicode_width;
use unicode_width::UnicodeWidthChar;

use std::io;
use std::io::Read;
use std::env;
use std::fmt;
use std::str::from_utf8;


fn main() {
    let args: Vec<_> = env::args().collect();
    match Options::getopts(&args[..]) {
        Ok(options)   => {
            let thing = io::stdin();
            let stdin = Chars { inner: thing.lock() };
            CharInfo::new(options, stdin).run();
        },
        Err(misfire)  => {
            println!("{}", misfire);
            env::set_exit_status(misfire.exit_status());
        },
    }
}



struct CharInfo<I> {
    options: Options,
    count: u64,

    input: Chars<I>,
}

impl<I: Read> CharInfo<I> {

    fn new(options: Options, iterator: Chars<I>) -> CharInfo<I> {
        CharInfo {
            count:    if options.bytes { 0 } else { 1 },
            options:  options,
            input:    iterator,
        }
    }

    fn run(mut self) {
        for input in self.input {
            match input {
                Ok(ReadChar::Ok(c, bytes)) => {
                    let char_type = CharType::of(c);

                    print_count(self.count);
                    print!("{}", char_type.style().paint(&number(c)));
                    print!(" {} ", Fixed(244).paint("="));

                    match bytes {
                        ReadBytes::FirstByte(b) => {
                            print_hex(b);
                            self.count += 1;
                        },

                        ReadBytes::WholeBuffer(buf, width) => {
                            print_buf(&buf[..width]);

                            if self.options.show_names {
                                if let Some(name) = unicode_names::name(c) {
                                    print!(" {}", Blue.paint(&format!("({})", name)));
                                }
                            }

                            self.count += if self.options.bytes { width as u64 }
                                                                   else { 1u64 };
                        },
                    }

                    print!("\n");
                },

                Ok(ReadChar::Invalid(bytes)) => {
                    print_count(self.count);
                    print!(" {}", Red.bold().paint("!!!"));
                    print!(" {} ", Red.paint("="));

                    match bytes {
                        ReadBytes::FirstByte(b) => {
                            print_hex(b);
                            self.count += 1;
                        },

                        ReadBytes::WholeBuffer(buf, width) => {
                            print_buf(&buf[..width]);
                            self.count += if self.options.bytes { width as u64 }
                                                                   else { 1u64 };
                        },
                    }

                    print!("\n");
                },

                Err(ref e) => {
                    println!("{}", e)
                },
            }
        }
    }
}

pub struct Chars<R> {
    inner: R,
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



struct Options {
    bytes:       bool,
    show_names:  bool,
}

impl Options {
    pub fn getopts(args: &[String]) -> Result<Options, Misfire> {
        let mut opts = getopts::Options::new();
        opts.optflag("b", "bytes",     "show count in number of bytes, not characters");
        opts.optflag("n", "names",     "show unicode name of each character");
        opts.optflag("",  "version",   "display version of program");
        opts.optflag("?", "help",      "show list of command-line options");

        let matches = match opts.parse(args) {
            Ok(m) => m,
            Err(e) => return Err(Misfire::InvalidOptions(e)),
        };

        if matches.opt_present("help") {
            return Err(Misfire::Help(opts.usage("Usage:\n  charinfo [options] < file")))
        }
        else if matches.opt_present("version") {
            return Err(Misfire::Version);
        }

        Ok(Options {
            bytes:       matches.opt_present("bytes"),
            show_names:  matches.opt_present("names"),
        })
    }
}



enum Misfire {
    InvalidOptions(getopts::Fail),
    Help(String),
    Version,
}

impl Misfire {
    pub fn exit_status(&self) -> i32 {
        if let Misfire::Help(_) = *self { 2 }
                                   else { 3 }
    }
}

impl fmt::Display for Misfire {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Misfire::InvalidOptions(ref e) => write!(f, "{}", e),
            Misfire::Help(ref text)        => write!(f, "{}", text),
            Misfire::Version               => write!(f, "charinfo {}", env!("CARGO_PKG_VERSION")),
        }
    }
}



#[derive(PartialEq)]
enum CharType {
    Normal,
    Combining,
    Control,
}

impl CharType {
    fn of(c: char) -> CharType {
        let num = c as u32;

        if c.is_control() {
            CharType::Control
        }
        else if num >= 0x300 && num < 0x370 {
            CharType::Combining
        }
        else {
            CharType::Normal
        }
    }

    fn style(&self) -> Style {
        match *self {
            CharType::Control    => Green.normal(),
            CharType::Combining  => Purple.normal(),
            CharType::Normal     => Plain,
        }
    }
}


fn print_count(count: u64) {
    print!("{}", Fixed(244).paint(&format!("{:>5}: ", count)));
}

fn number(c: char) -> String {
    let number = c as u32;

    if number <= 9 {
        format!(" #{} ", number)
    }
    else if number as u32 <= 31 {
        format!(" #{}", number)
    }
    else if number >= 0x300 && number < 0x370 {
        format!(" ' {}'", c)
    }
    else if UnicodeWidthChar::width(c) == Some(1) {
        format!(" '{}'", c)
    }
    else {
        format!("'{}'", c)
    }
}

fn print_hex(c: u8) {
    print!("{:0>2x}", c as u8);
}

fn print_buf(buf: &[u8]) {
    print_hex(buf[0]);

    for index in 1 .. buf.len() {
        print!(" ");
        print_hex(buf[index]);
    }
}
