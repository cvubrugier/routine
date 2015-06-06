//
// (C) Copyright 2015
// Christophe Vu-Brugier <cvubrugier@fastmail.fm>
//
// SPDX-License-Identifier:	MIT
//

extern crate getopts;
extern crate time;
extern crate piston;
extern crate graphics;
extern crate piston_window;
extern crate glutin_window;
extern crate gfx_graphics;

use getopts::Options;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;
use time::{ Duration, SteadyTime };

use piston::window::WindowSettings;
use piston_window::*;
use gfx_graphics::GlyphCache;
use glutin_window::{ GlutinWindow, OpenGL };

type Color = [f32; 4];
const RED:   Color = [0.8, 0.0, 0.0, 1.0];
const GREEN: Color = [0.0, 0.8, 0.0, 1.0];
const BLUE:  Color = [0.0, 0.0, 0.8, 1.0];
const WHITE: Color = [1.0, 1.0, 1.0, 1.0];

enum StepId {
    Prep,
    Work,
    Rest,
}

struct Step {
    id: StepId,
    name: String,
    color: Color,
    duration: Duration,
}

const NR_STEPS: usize = 3;

#[derive(Debug)]
enum ParseError {
    HelpArgument,
    InvalidArgument,
    InvalidFormat,
}

struct Settings {
    prep_duration: Duration,
    work_duration: Duration,
    rest_duration: Duration,
}

impl Settings {

    fn new(prep_seconds: i64,
           work_seconds: i64,
           rest_seconds: i64) -> Settings {
        Settings {
            prep_duration: Duration::seconds(prep_seconds),
            work_duration: Duration::seconds(work_seconds),
            rest_duration: Duration::seconds(rest_seconds),
        }
    }

    fn set_from_cmdline(&mut self) -> Result<(), ParseError> {
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
                match i64::from_str(&*argval) {
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

struct App {
    steps: [Step; NR_STEPS],
    step_idx: usize,
    expiration: SteadyTime,
    round_nr: u32,
}

impl App {
    fn new(settings: &Settings) -> App {
        App {
            steps: [
                Step {
                    id: StepId::Prep,
                    name: "Prepare".to_string(),
                    color: BLUE,
                    duration: settings.prep_duration,
                },
                Step {
                    id: StepId::Work,
                    name: "Work".to_string(),
                    color: RED,
                    duration: settings.work_duration,
                },
                Step {
                    id: StepId::Rest,
                    name: "Rest".to_string(),
                    color: GREEN,
                    duration: settings.rest_duration,
                },
            ],
            step_idx: 0,
            expiration: SteadyTime::now() + settings.prep_duration,
            round_nr: 1,
        }
    }

    fn tick(&mut self) -> (u32, &Step, Duration) {
        let mut remaining = self.expiration - SteadyTime::now();

        if remaining < Duration::zero() {
            // The current step is done. Increment the number of
            // rounds if it was the last step and pick the next step:
            // it should not be a "prepare" step (only done once) and
            // should have a non zero duration.
            loop {
                self.step_idx += 1;
                if self.step_idx >= self.steps.len() {
                    self.round_nr += 1;
                    self.step_idx = self.step_idx % self.steps.len();
                }
                if let StepId::Prep = self.steps[self.step_idx].id {
                    // Skip the "prepare" step: it is only done once
                    continue;
                }
                remaining = self.steps[self.step_idx].duration;
                if remaining.is_zero() {
                    continue;
                }
                break;
            }
            self.expiration = SteadyTime::now() + remaining;
        }

        (self.round_nr, &self.steps[self.step_idx], remaining)
    }
}

fn main() {
    let glutin_window = Rc::new(RefCell::new(
        GlutinWindow::new(
            OpenGL::_3_2,
            WindowSettings::new("Routine", [640, 480])
                .exit_on_esc(true)
         )
    ));

    let mut settings = Settings::new(10, 30, 30);
    if let Err(e) = settings.set_from_cmdline() {
        match e {
            ParseError::InvalidArgument => {
                println!("Error: invalid argument.");
                std::process::exit(1);
            }
            ParseError::InvalidFormat => {
                println!("Error: invalid argument format.");
                std::process::exit(1);
            }
            ParseError::HelpArgument => {
                return;
            }
        }
    }

    let app = Rc::new(RefCell::new(
        App::new(&settings)
    ));

    let events = PistonWindow::new(glutin_window, app);
    let ref font = Path::new("assets/fonts/FiraMono-Medium.ttf");
    let factory = events.factory.borrow().clone();
    let mut glyph_cache = GlyphCache::new(font, factory).unwrap();

    for e in events {
        let mut a = e.app.borrow_mut();
        let (round_nr, step, remaining) = a.tick();

        e.draw_2d(|c, g| {
            use graphics::*;

            let label = format!("Round {} | {}", round_nr, step.name);
            let time_remaining = format!("{:02}:{:02}.{:01}",
                                         remaining.num_minutes(),
                                         remaining.num_seconds() % 60,
                                         (remaining.num_milliseconds() / 100) % 10);

            clear(step.color, g);

            text::Text::colored(WHITE, 40).draw(
                &*label,
                &mut glyph_cache,
                &c.draw_state,
                c.transform.trans(70.0, 150.0),
                g
            );

            text::Text::colored(WHITE, 120).draw(
                &*time_remaining,
                &mut glyph_cache,
                &c.draw_state,
                c.transform.trans(60.0, 300.0),
                g
            );
        });
    }
}
