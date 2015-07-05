#![feature(str_char, unicode)]

extern crate getopts;
extern crate ansi_term;
use ansi_term::Colour::*;
use ansi_term::Style;

extern crate rustc_unicode;
extern crate unicode_names;
extern crate unicode_normalization;
extern crate unicode_width;

use std::io;
use std::io::Read;
use std::env;
use std::fmt;
use std::process;

mod iter;
use iter::{Chars, ReadBytes, ReadChar};

mod char;
use char::{CharType, CharExt};

mod scripts;


fn main() {
    let args: Vec<_> = env::args().collect();
    match Options::getopts(&args[..]) {
        Ok(options)   => {
            let thing = io::stdin();
            let stdin = Chars { inner: thing.lock() };
            Charmander::new(options, stdin).run();
        },
        Err(misfire)  => {
            println!("{}", misfire);
            process::exit(misfire.exit_status());
        },
    }
}


struct Charmander<I> {
    options: Options,
    count: u64,

    input: Chars<I>,
}

impl<I: Read> Charmander<I> {

    fn new(options: Options, iterator: Chars<I>) -> Charmander<I> {
        Charmander {
            count:    if options.bytes { 0 } else { 1 },
            options:  options,
            input:    iterator,
        }
    }

    fn run(mut self) {
        for input in self.input {
            match input {
                Ok(ReadChar::Ok(c, bytes)) => {
                    let style = match c.char_type() {
                        CharType::Control    => Green.normal(),
                        CharType::Combining  => Purple.normal(),
                        CharType::Normal     => Style::default(),
                    };

                    print_count(self.count);
                    print!("{}", style.paint(&number(c)));
                    print!(" {} ", Fixed(244).paint("="));

                    match bytes {
                        ReadBytes::FirstByte(b) => {
                            print_hex(b);

                            if self.options.show_scripts {
                                if let Some(script) = c.script() {
                                    print!(" {}", Purple.paint(&format!("[{}]", script.name())));
                                }
                            }

                            self.count += 1;
                        },

                        ReadBytes::WholeBuffer(buf, width) => {
                            print_buf(&buf[..width]);

                            if self.options.show_names {
                                if let Some(name) = unicode_names::name(c) {
                                    print!(" {}", Blue.paint(&format!("({})", name)));
                                }
                            }

                            if self.options.show_scripts {
                                if let Some(script) = c.script() {
                                    print!(" {}", Purple.paint(&format!("[{}]", script.name())));
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
    else if c.is_multicolumn() {
        format!("'{}'", c)
    }
    else {
        format!(" '{}'", c)
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


struct Options {
    bytes:         bool,
    show_names:    bool,
    show_scripts:  bool,
}

impl Options {
    pub fn getopts(args: &[String]) -> Result<Options, Misfire> {
        let mut opts = getopts::Options::new();
        opts.optflag("b", "bytes",     "show count in number of bytes, not characters");
        opts.optflag("n", "names",     "show unicode name of each character");
        opts.optflag("s", "scripts",   "show script for each chararcter");
        opts.optflag("",  "version",   "display version of program");
        opts.optflag("?", "help",      "show list of command-line options");

        let matches = match opts.parse(args) {
            Ok(m) => m,
            Err(e) => return Err(Misfire::InvalidOptions(e)),
        };

        if matches.opt_present("help") {
            return Err(Misfire::Help(opts.usage("Usage:\n  charm [options] < file")))
        }
        else if matches.opt_present("version") {
            return Err(Misfire::Version);
        }

        Ok(Options {
            bytes:         matches.opt_present("bytes"),
            show_names:    matches.opt_present("names"),
            show_scripts:  matches.opt_present("scripts"),
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
            Misfire::Version               => write!(f, "charm {}", env!("CARGO_PKG_VERSION")),
        }
    }
}
