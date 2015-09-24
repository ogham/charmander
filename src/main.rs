//! charmander, a character-viewing program

#![feature(str_char, unicode)]

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(unused_results)]

extern crate getopts;
extern crate ansi_term;
use ansi_term::Colour::*;
use ansi_term::Style;

extern crate rustc_unicode;
extern crate unicode_names;
extern crate unicode_normalization;
extern crate unicode_width;

use std::fs::File;
use std::io::{stdin, Read};
use std::env;
use std::process;

mod iter;
use iter::{Chars, ReadBytes, ReadChar};

mod char;
use char::{DisplayType, CharExt};

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
                let stdin = stdin();
                let iterator = Chars::new(stdin.lock());
                Charmander::new(options.flags, iterator).run();
            }
        },
        Err(misfire) => {
            println!("{}", misfire);
            process::exit(misfire.exit_status());
        },
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

                    // Display certain types of character in a colour.
                    // (these colours are pretty much arbitrary)
                    let style = match c.char_type() {
                        DisplayType::Control    => Green.normal(),
                        DisplayType::Combining  => Purple.normal(),
                        DisplayType::Normal     => Style::default(),
                    };

                    print_count(self.count);
                    print!("{} {} ", style.paint(&number(c)), Fixed(244).paint("="));

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
    else if number <= 31 {
        format!(" #{}", number)
    }
    else if number >= 0x300 && number < 0x370 {
        format!(" '\u{25CC}{}'", c)
    }
    else if c.is_multicolumn() {
        format!("'{}'", c)
    }
    else {
        format!(" '{}'", c)
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
