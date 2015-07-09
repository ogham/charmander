#![feature(str_char, unicode)]

extern crate getopts;
extern crate ansi_term;
use ansi_term::Colour::*;
use ansi_term::Style;

extern crate rustc_unicode;
extern crate unicode_names;
extern crate unicode_normalization;
extern crate unicode_width;

use std::fs::File;
use std::io;
use std::io::Read;
use std::env;
use std::process;

mod iter;
use iter::{Chars, ReadBytes, ReadChar};

mod char;
use char::{CharType, CharExt};

mod options;
use options::{Options, Flags};

mod scripts;


fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    match Options::getopts(&args[..]) {
        Ok(options) => {
            if let Some(file_name) = options.input_file_name {
                match File::open(file_name.clone()) {
                    Ok(f)  => {
                        Charmander::new(options.flags, Chars::new(f)).run();
                    },
                    Err(e) => {
                        println!("Couldn't open `{}` for reading: {}", &*file_name, e);
                        process::exit(1);
                    },
                }
            }
            else {
                let thing = io::stdin();
                let stdin = Chars { inner: thing.lock() };
                Charmander::new(options.flags, stdin).run();
            }
        },
        Err(misfire) => {
            println!("{}", misfire);
            process::exit(misfire.exit_status());
        },
    }
}


struct Charmander<I> {
    flags: Flags,
    count: u64,

    input: Chars<I>,
}

impl<I: Read> Charmander<I> {

    fn new(flags: Flags, iterator: Chars<I>) -> Charmander<I> {
        Charmander {
            count:  if flags.bytes { 0 } else { 1 },
            flags:  flags,
            input:  iterator,
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

                            if self.flags.show_scripts {
                                if let Some(script) = c.script() {
                                    print!(" {}", Purple.paint(&format!("[{}]", script.name())));
                                }
                            }

                            self.count += 1;
                        },

                        ReadBytes::WholeBuffer(buf, width) => {
                            print_buf(&buf[..width]);

                            if self.flags.show_names {
                                if let Some(name) = unicode_names::name(c) {
                                    print!(" {}", Blue.paint(&format!("({})", name)));
                                }
                            }

                            if self.flags.show_scripts {
                                if let Some(script) = c.script() {
                                    print!(" {}", Purple.paint(&format!("[{}]", script.name())));
                                }
                            }

                            self.count += if self.flags.bytes { width as u64 }
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
                            self.count += if self.flags.bytes { width as u64 }
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
