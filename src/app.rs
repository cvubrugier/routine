// (C) Copyright 2016
// Christophe Vu-Brugier <cvubrugier@fastmail.fm>
//
// SPDX-License-Identifier:	MIT
//

extern crate time;

use piston::input::RenderArgs;
use graphics;
use graphics::types::Color;
use opengl_graphics::GlGraphics;
use opengl_graphics::glyph_cache::GlyphCache;
use self::time::{Duration, SteadyTime};
use settings;

const RED: Color = [0.8, 0.0, 0.0, 1.0];
const GREEN: Color = [0.0, 0.8, 0.0, 1.0];
const BLUE: Color = [0.0, 0.0, 0.8, 1.0];
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

pub struct App {
    steps: [Step; NR_STEPS],
    step_idx: usize,
    expiration: SteadyTime,
    round_nr: u32,
}

impl App {
    pub fn new(settings: settings::Settings) -> App {
        App {
            steps: [Step {
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
                    }],
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

    pub fn on_render(&mut self, args: &RenderArgs, gl: &mut GlGraphics, cache: &mut GlyphCache) {
        let (round_nr, step, remaining) = self.tick();

        gl.draw(args.viewport(), |c, g| {
            use graphics::*;
            let label = format!("Round {} | {}", round_nr, step.name);
            let time_remaining = format!("{:02}:{:02}.{:01}",
                                         remaining.num_minutes(),
                                         remaining.num_seconds() % 60,
                                         (remaining.num_milliseconds() / 100) % 10);

            clear(step.color, g);

            let text_label = graphics::Text::new_color(WHITE, 40);
            text_label.draw(&*label,
                            cache,
                            &c.draw_state,
                            c.transform.trans(70.0, 150.0),
                            g);

            let text_timer = graphics::Text::new_color(WHITE, 120);
            text_timer.draw(&*time_remaining,
                            cache,
                            &c.draw_state,
                            c.transform.trans(60.0, 300.0),
                            g);
        });
    }
}
