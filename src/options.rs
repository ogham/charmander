use std::fmt;

use getopts;


pub struct Options {
    pub flags: Flags,
    pub input_file_name: Option<String>,
}

pub struct Flags {
    pub bytes:           bool,
    pub show_names:      bool,
    pub show_scripts:    bool,
}

static USAGE: &'static str = "Usage:\n  charm [options] file";

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
            return Err(Misfire::Help(opts.usage(USAGE)))
        }
        else if matches.opt_present("version") {
            return Err(Misfire::Version);
        }

        let input_file_name = match matches.free.len() {
            0 => None,
            1 => Some(matches.free[0].clone()),
            _ => return Err(Misfire::Help(opts.usage(USAGE))),
        };

        Ok(Options {
            flags: Flags {
                bytes:           matches.opt_present("bytes"),
                show_names:      matches.opt_present("names"),
                show_scripts:    matches.opt_present("scripts"),
            },
            input_file_name: input_file_name,
        })
    }
}


pub enum Misfire {
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
