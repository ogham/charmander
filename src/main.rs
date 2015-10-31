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

use std::error::Error;
use std::fs::File;
use std::io::{stdin, Read};

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

    let app = Charmander {
        // Humans start counting things from 1, but the offset of each
        // character needs to start from 0.
        count:  if flags.bytes { 0 } else { 1 },
        flags:  flags,
    };

    if let Some(file_name) = matches.value_of("input_file") {
        match File::open(file_name.clone()) {
            Ok(f)  => app.run(f),
            Err(e) => error_and_exit(file_name, e),
        }
    }
    else {
        let stdin = stdin();
        app.run(stdin.lock());
    }
}

/// Display an error that has something to do with the given filename,
/// then exit the program immediately with failure.
fn error_and_exit<E: Error>(file_name: &str, error: E) {
    use std::process::exit;

    println!("{}: {}: {}", program_name(), file_name, error);
    exit(1);
}

/// Returns the string representing the path that this program was
/// invocated by. Usually this will be 'charm' -- it's the first
/// 'argument' in the arguments list. If for some reason it can't be
/// found, 'charm' will be used instead, but honestly if the arguments
/// list is empty then something has gone badly wrong somewhere.
fn program_name() -> String {
    use std::env::args;
    args().next().unwrap_or_else(|| "charm".to_owned())
}


struct Charmander {

    /// Flags that affect the output.
    flags: Flags,

    /// The count to display next to each character.
    count: u64,
}

impl Charmander {
    fn run<I: Read>(mut self, char_stream: I) {
        for read_char in Chars::new(char_stream) {
            match read_char {
                Ok(ReadChar::Ok(c, bytes)) => {

                    self.print_count();
                    print!("{}\t{} ", self.number(c), Fixed(244).paint("="));

                    match bytes {
                        ReadBytes::FirstByte(b) => {
                            self.print_hex(b);
                            self.count += 1;
                        },

                        ReadBytes::WholeBuffer(buf, width) => {
                            self.print_buf(&buf[..width]);
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
                    self.print_count();
                    print!("{}\t{} ", Red.bold().paint("!!!"), Fixed(244).paint("="));

                    match bytes {
                        ReadBytes::FirstByte(b) => {
                            self.print_hex(b);
                            self.count += 1;
                        },

                        ReadBytes::WholeBuffer(buf, width) => {
                            self.print_buf(&buf[..width]);
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

    fn print_count(&self) {
        print!("{}", Fixed(244).paint(&format!("{:>5}: ", self.count)));
    }

    fn number(&self, c: char) -> String {
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

    fn print_hex(&self, c: u8) {
        print!("{:0>2x}", c);
    }

    fn print_buf(&self, buf: &[u8]) {
        self.print_hex(buf[0]);

        for index in 1 .. buf.len() {
            print!(" ");
            self.print_hex(buf[index]);
        }
    }
}

