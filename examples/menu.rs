
extern crate glutin;
extern crate image;
extern crate clock_ticks;
extern crate uil;

use std::old_io::BufReader;
use glium::{DisplayBuild, Surface};
use uil::layout;
use uil::rendering;
use std::old_io::timer;
use std::time::duration::Duration;



fn main() {

    //////////////////////////////////////////////////////////////////////////////
    // uil related code
    //
    let library = {
        let file = File::open(&Path::new("./examples/menu.markup")).unwrap();
        let reader = BufferedReader::new(file);
        uil::markup::parse(uil::StdOutErrorReporter, reader)
    };

    let styledefs = {
        let file = File::open(&Path::new("./examples/menu.deps")).unwrap();
        let reader = BufferedReader::new(file);
        uil::deps::parse(uil::StdOutErrorReporter, reader)
    };

    let stylesheet = {
        let file = File::open(&Path::new("./examples/menu.style")).unwrap();
        let reader = BufferedReader::new(file);
        uil::style::parse(uil::StdOutErrorReporter, reader, &styledefs)
    };

    let router = uil::Router::with_


    //////////////////////////////////////////////////////////////////////////////
    // glium related code
    //
    let display = glutin::WindowBuilder::new()
        .with_vsync()
        .build_glium()
        .unwrap();

    let image = image::load(BufReader::new(include_bytes!("./btn.png")),
        image::PNG).unwrap();

    // Use of the "use_glium" feature.
    let renderer = uil::glium::GliumRenderer::new(display, image);

    //////////////////////////////////////////////////////////////////////////////
    // main loop (modified example from glium lib)
    //
    start_loop(|| {

        // TODO: FIXME call router.render()

        // polling and handling the events received by the window
        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => return Action::Stop,
                _ => ()
            }
        }

        Action::Continue
    });
}

pub enum Action {
    Stop,
    Continue,
}

pub fn start_loop<F>(mut callback: F) where F: FnMut() -> Action {
    let mut accumulator = 0;
    let mut previous_clock = clock_ticks::precise_time_ns();
    loop {
        match callback() {
            Action::Stop => break,
            Action::Continue => ()
        };
        let now = clock_ticks::precise_time_ns();
        accumulator += now - previous_clock;
        previous_clock = now;
        const FIXED_TIME_STAMP: u64 = 16666667;
        while accumulator >= FIXED_TIME_STAMP {
            accumulator -= FIXED_TIME_STAMP;
            // if you have a game, update the state here
        }
        timer::sleep(Duration::nanoseconds((FIXED_TIME_STAMP - accumulator) as i64));
    }
}
