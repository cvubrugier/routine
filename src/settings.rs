//
// (C) Copyright 2016
// Christophe Vu-Brugier <cvubrugier@fastmail.fm>
//
// SPDX-License-Identifier:	MIT
//

extern crate getopts;
extern crate time;

use self::getopts::Options;
use std;
use self::time::Duration;

#[derive(Debug)]
pub enum ParseError {
    HelpArgument,
    InvalidArgument,
    InvalidFormat,
}

pub struct Settings {
    pub prep_duration: Duration,
    pub work_duration: Duration,
    pub rest_duration: Duration,
}

impl Settings {

    pub fn new(prep_seconds: i64,
           work_seconds: i64,
           rest_seconds: i64) -> Settings {
        Settings {
            prep_duration: Duration::seconds(prep_seconds),
            work_duration: Duration::seconds(work_seconds),
            rest_duration: Duration::seconds(rest_seconds),
        }
    }

    pub fn set_from_cmdline(&mut self) -> Result<(), ParseError> {
        let args: Vec<String> = std::env::args().collect();

        let mut opts = Options::new();
        opts.optflag("h", "help", "display this help and exit");
        opts.optopt("p", "prep", "set the preparation time to NUMBER seconds", "NUMBER");
        opts.optopt("w", "work", "set the workout time to NUMBER seconds", "NUMBER");
        opts.optopt("r", "rest", "set the rest time to NUMBER seconds", "NUMBER");

        let matches = match opts.parse(&args[1..]) {
            Ok(m) => { m }
            Err(_) => { return Err(ParseError::InvalidArgument) }
        };

        if matches.opt_present("h") {
            let brief = format!("Usage: {} [options]", args[0]);
            print!("{}", opts.usage(&brief));
            return Err(ParseError::HelpArgument);
        };

        for arg in vec!["p", "w", "r"] {
            match self.set_arg(&matches, arg) {
                Ok(_) => {},
                Err(e) => { return Err(e) },
            }
        }
        Ok(())
    }

    fn set_arg(&mut self,
               matches: &getopts::Matches,
               arg: &str) -> Result<(), ParseError> {
        match matches.opt_str(arg) {
            Some(argval) => {
                match argval.parse::<i64>() {
                    Ok(v) => {
                        match arg {
                            "p" => { self.prep_duration = Duration::seconds(v) },
                            "w" => { self.work_duration = Duration::seconds(v) },
                            "r" => { self.rest_duration = Duration::seconds(v) },
                            _ => { unreachable!() }
                        }
                        Ok(())
                    },
                    Err(_) => Err(ParseError::InvalidFormat),
                }
            },
            None => Ok(()),
        }
    }

}
