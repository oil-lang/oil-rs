
use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::thread;

use glium::DisplayBuild;
use uil::RenderBackbend;
use uil;
use glutin;
use clock_ticks;

pub fn run_example(title: &str, markup_path: &str, deps_path: &str, style_path: &str) {

    //////////////////////////////////////////////////////////////////////////////
    // uil related code
    //
    let library = {
        let file = File::open(&Path::new(markup_path)).unwrap();
        let reader = BufReader::new(file);
        let mut unlinked = uil::markup::parse(uil::StdOutErrorReporter, reader);
        unlinked.resolve_templates();
        unlinked
    };

    let defs = uil::deps::parse_file(uil::StdOutErrorReporter, deps_path);

    //////////////////////////////////////////////////////////////////////////////
    // glium display start
    //
    let display = glutin::WindowBuilder::new()
        .with_vsync()
        .with_title(title.to_string())
        .build_glium()
        .unwrap();

    //////////////////////////////////////////////////////////////////////////////
    // uil resource manager and final tree
    //

    let mut resource_manager = uil::resource::create_resource_manager(&display);

    let stylesheet = {
        let file = File::open(&Path::new(style_path)).unwrap();
        let reader = BufReader::new(file);
        uil::style::parse(uil::StdOutErrorReporter, reader, &defs, &mut resource_manager)
    };

    let (width, height) = display.get_window().unwrap().get_inner_size().unwrap();

    let mut renderer = uil::rendering::backend::GliumRenderer::new(&display);
    let mut router = uil::Router::from_library_and_stylesheet(
        &display,
        &resource_manager,
        library,
        &stylesheet
    );
    let data_binder_context = uil::DataBinderContext::default();

    //////////////////////////////////////////////////////////////////////////////
    // main loop (modified example from glium lib)
    //
    start_loop(|| {

        let vp = uil::Viewport { width: width as f32, height: height as f32 };

        // Update views
        router.update(&display, &resource_manager, vp, &data_binder_context);

        // Render views
        let mut f = renderer.prepare_frame(vp);
        router.render_views(&renderer, &mut f, &resource_manager);
        renderer.flush_frame(f);

        // polling and handling the events received by the window
        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => return Action::Stop,
                glutin::Event::KeyboardInput(
                    glutin::ElementState::Pressed,
                    _,
                    Some(vkc)
                ) => {
                    match vkc {
                        glutin::VirtualKeyCode::Left => router.focus_left(),
                        glutin::VirtualKeyCode::Right => router.focus_right(),
                        glutin::VirtualKeyCode::Up => router.focus_up(),
                        glutin::VirtualKeyCode::Down => router.focus_down(),
                        _ => ()
                    }
                }
                _ => ()
            }
        }

        Action::Continue
    });
}

enum Action {
    Stop,
    Continue,
}

fn start_loop<F>(mut callback: F) where F: FnMut() -> Action {
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
        }

        thread::sleep_ms(((FIXED_TIME_STAMP - accumulator) / 1000000) as u32);
    }
}
