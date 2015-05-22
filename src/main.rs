#![feature(exit_status, str_char, unicode)]

extern crate getopts;
extern crate term;
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
        let mut t = term::stdout().unwrap();
        for input in self.input {
            match input {
                Ok(ReadChar::Ok(c, buf, width)) => {
                    let char_type = CharType::of(c);

                    if char_type == CharType::Control {
                        t.fg(term::color::GREEN).unwrap();
                    }
                    else if char_type == CharType::Combining {
                        t.fg(term::color::MAGENTA).unwrap();
                    }

                    print!("{:>5}: {} = {}", self.count, CharDisplay(c), NumDisplay(&buf[..width]));

                    if self.options.show_names {
                        if let Some(name) = unicode_names::name(c) {
                            print!(" ({})", name);
                        }
                    }

                    if char_type != CharType::Normal {
                        t.reset().unwrap();
                    }

                    self.count += if self.options.bytes { width as u64 }
                                                           else { 1u64 };
                    print!("\n");
                },

                Ok(ReadChar::ImmediateOk(c)) => {
                    println!("{:>5}: {} = {:0>2x}", self.count, CharDisplay(c), c as u8);
                    self.count += 1;
                },

                Ok(ReadChar::ImmediateInvalid(first)) => {
                    t.fg(term::color::BRIGHT_RED).unwrap();
                    println!("{:>5}:  !!! = {:0>2x}", self.count, first);
                    self.count += 1;
                },
                Ok(ReadChar::Invalid(buf, width)) => {
                    t.fg(term::color::BRIGHT_RED).unwrap();
                    println!("{:>5}:  !!! = {}", self.count, NumDisplay(&buf[..width]));
                    self.count += width as u64;
                },
                Err(ref e) => {
                    println!("{}", e)
                },
            }

            t.reset().unwrap();
        }
    }
}

pub struct Chars<R> {
    inner: R,
}

pub enum ReadChar {
    Ok(char, [u8; 4], usize),
    ImmediateOk(char),
    Invalid([u8; 4], usize),
    ImmediateInvalid(u8),
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

        let width = match utf8_char_width(first_byte) {
            0 => return Some(Ok(ReadChar::ImmediateInvalid(first_byte))),
            1 => return Some(Ok(ReadChar::ImmediateOk(first_byte as char))),
            w => w,
        };

        assert! { width <= 4 };

        let mut buf = [first_byte, 0, 0, 0];
        let mut start = 1;

        while start < width {
            match self.inner.read(&mut buf[start..width]) {
                Ok(0)   => return Some(Ok(ReadChar::Invalid(buf, width))),
                Ok(n)   => start += n,
                Err(e)  => return Some(Err(e)),
            }
        }

        match from_utf8(&buf[..width]) {
            Ok(s)  => Some(Ok(ReadChar::Ok(s.char_at(0), buf, width))),
            Err(_) => Some(Ok(ReadChar::Invalid(buf, width))),
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
}



struct CharDisplay(char);

impl fmt::Display for CharDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let number = self.0 as u32;

        if number <= 9 {
            write!(f, " #{} ", number)
        }
        else if number as u32 <= 31 {
            write!(f, " #{}", number)
        }
        else if number >= 0x300 && number < 0x370 {
            write!(f, " ' {}'", self.0)
        }
        else if UnicodeWidthChar::width(self.0) == Some(1) {
            write!(f, " '{}'", self.0)
        }
        else {
            write!(f, "'{}'", self.0)
        }
    }
}


struct NumDisplay<'buf>(&'buf [u8]);

impl<'buf> fmt::Display for NumDisplay<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{:0>2x}", &self.0[0]));

        for index in 1 .. self.0.len() {
            try!(write!(f, " {:0>2x}", &self.0[index]));
        }

        Ok(())
    }
}
