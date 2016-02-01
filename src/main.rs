//
// (C) Copyright 2015
// Christophe Vu-Brugier <cvubrugier@fastmail.fm>
//
// SPDX-License-Identifier:	MIT
//

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate piston_window;

use piston_window::WindowSettings;
use piston::input::RenderEvent;
use piston::event_loop::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use opengl_graphics::glyph_cache::GlyphCache;
use std::path::Path;

mod app;
mod settings;

fn main() {
    let opengl = OpenGL::V3_2;
    let mut window: Window = WindowSettings::new(
        "Routine",
        [640, 480]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();
    // Must be declared after the window is created
    let ref mut gl = GlGraphics::new(opengl);

    let mut settings = settings::Settings::new(10, 30, 30);
    if let Err(e) = settings.set_from_cmdline() {
        match e {
            settings::ParseError::InvalidArgument => {
                println!("Error: invalid argument.");
                std::process::exit(1);
            }
            settings::ParseError::InvalidFormat => {
                println!("Error: invalid argument format.");
                std::process::exit(1);
            }
            settings::ParseError::HelpArgument => {
                return;
            }
        }
    }

    let mut app = app::App::new(settings);

    let font_path = Path::new("assets/fonts/FiraMono-Medium.ttf");
    let ref mut glyph_cache = GlyphCache::new(font_path).unwrap();

    let mut events = window.events();
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.on_render(&args, gl, glyph_cache);
        }
    }
}
