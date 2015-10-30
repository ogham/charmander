//! charmander, a character-viewing program

#![feature(str_char, unicode)]

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(unused_results)]

#[macro_use]
extern crate clap;
use clap::App;

extern crate ansi_term;
use ansi_term::Colour::*;

extern crate rustc_unicode;
extern crate unicode_names;
extern crate unicode_normalization;
extern crate unicode_width;
use unicode_width::UnicodeWidthChar;

use std::fs::File;
use std::io::{stdin, Read};
use std::process;

mod iter;
use iter::{Chars, ReadBytes, ReadChar};

mod char;
use char::{CharExt};

mod scripts;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Flags {
    pub bytes:           bool,
    pub show_names:      bool,
    pub show_scripts:    bool,
    pub show_widths:     bool,
}

fn main() {
    let yaml = load_yaml!("args.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let flags = Flags {
        bytes:           matches.is_present("bytes"),
        show_names:      matches.is_present("names"),
        show_scripts:    matches.is_present("scripts"),
        show_widths:     matches.is_present("widths"),
    };

    if let Some(file_name) = matches.value_of("input_file") {
        match File::open(file_name.clone()) {
            Ok(f)  => {
                Charmander::new(flags, Chars::new(f)).run();
            },
            Err(e) => {
                println!("Couldn't open `{}` for reading: {}", &*file_name, e);
                process::exit(1);
            },
        }
    }
    else {
        let stdin = stdin();
        let iterator = Chars::new(stdin.lock());
        Charmander::new(flags, iterator).run();
    }
}

/// The main program body. It's able to run on anything that fits in the `Chars` iterator.
struct Charmander<I> {

    /// Flags that affect the output.
    flags: Flags,

    /// The count to display next to each character.
    count: u64,

    /// The iterator to loop through.
    input: Chars<I>,
}

impl<I: Read> Charmander<I> {

    fn new(flags: Flags, iterator: Chars<I>) -> Charmander<I> {
        Charmander {
            // Humans start counting things from 1, but the offset of each
            // character needs to start from 0.
            count:  if flags.bytes { 0 } else { 1 },
            flags:  flags,
            input:  iterator,
        }
    }

    fn run(mut self) {
        for input in self.input {
            match input {
                Ok(ReadChar::Ok(c, bytes)) => {

                    print_count(self.count);
                    print!("{}\t{} ", number(c), Fixed(244).paint("="));

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

                    if self.flags.show_widths {
                        if let Some(width) = c.width() {
                            print!(" {}", Cyan.paint(&format!("<{}>", width)));
                        }
                        else {
                            print!(" {}", Cyan.paint("<C>"));
                        }
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

    if number <= 31 {
        let s = format!("#{}", number);
        Green.paint(&s).to_string()
    }
    else if c.is_combining() {
        let s = format!("◌{}", c);
        Red.paint(&s).to_string()
    }
    else if let Some(0) = c.width() {
        let s = format!(" {}", c);
        Cyan.paint(&s).to_string()
    }
    else {
        c.to_string()
    }
}

fn print_hex(c: u8) {
    print!("{:0>2x}", c);
}

fn print_buf(buf: &[u8]) {
    print_hex(buf[0]);

    for index in 1 .. buf.len() {
        print!(" ");
        print_hex(buf[index]);
    }
}
