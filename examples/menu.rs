#![feature(old_io, std_misc)]

extern crate glutin;
extern crate glium;
extern crate image;
extern crate clock_ticks;
extern crate uil;

use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use glium::DisplayBuild;
use std::old_io::timer;
use std::time::duration::Duration;

use uil::RenderBackbend;

fn main() {

    //////////////////////////////////////////////////////////////////////////////
    // uil related code
    //
    let library = {
        let file = File::open(&Path::new("./examples/menu.markup")).unwrap();
        let reader = BufReader::new(file);
        uil::markup::parse(uil::StdOutErrorReporter, reader)
    };

    let defs = uil::deps::parse_file(uil::StdOutErrorReporter, "./examples/menu.deps");

    //////////////////////////////////////////////////////////////////////////////
    // glium display start
    //
    let display = glutin::WindowBuilder::new()
        .with_vsync()
        .with_title("uil - menu example".to_string())
        .build_glium()
        .unwrap();

    //////////////////////////////////////////////////////////////////////////////
    // uil resource manager and final tree
    //

    let mut resource_manager = uil::ResourceManager::new(&display);

    let stylesheet = {
        let file = File::open(&Path::new("./examples/menu.style")).unwrap();
        let reader = BufReader::new(file);
        uil::style::parse(uil::StdOutErrorReporter, reader, &defs, &mut resource_manager)
    };

    let (width, height) = display.get_window().unwrap().get_inner_size().unwrap();

    let mut renderer = uil::backend::GliumRenderer::new(&display);
    let mut router = uil::Router::from_library_and_stylesheet(
        &display,
        &resource_manager,
        library,
        &stylesheet
    );

    //////////////////////////////////////////////////////////////////////////////
    // main loop (modified example from glium lib)
    //
    start_loop(|| {

        let vp = uil::Viewport { width: width as f32, height: height as f32 };

        // Update views
        router.update(&display, vp);

        // Render views
        let mut f = renderer.prepare_frame(vp);
        router.render_views(&renderer, &mut f, &resource_manager);
        renderer.flush_frame(f);

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
