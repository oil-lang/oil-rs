

#[macro_use]
extern crate glium;
extern crate glutin;
extern crate uil;
extern crate image;
extern crate clock_ticks;
extern crate uil;

use std::old_io::BufReader;
use glium::{DisplayBuild, Surface};
use std::old_io::timer;
use std::time::duration::Duration;

#[derive(Copy)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

struct GliumRenderer<'a> {
    display: &'a display,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer: glium::IndexBuffer,
    texture: glium::texture::CompressedTexture2d,
    program: glium::Program,
}

impl<'a> GliumRenderer<'a> {
    pub fn new(display: &'a Display) -> GliumRenderer<'a> {
        let image: image::load(BufReader::new(include_bytes!("./btn.png")),
            image::PNG).unwrap();

        let program = glium::Program::from_source(&display, r"
            #version 110

            uniform mat4 matrix;

            attribute vec2 position;
            attribute vec2 tex_coords;

            varying vec2 v_tex_coords;

            void main() {
                gl_Position = matrix * vec4(position, 0.0, 1.0);
                v_tex_coords = tex_coords;
            }
        ", r"
            #version 110
            uniform sampler2D texture;
            varying vec2 v_tex_coords;

            void main() {
                gl_FragColor = texture2D(texture, v_tex_coords);
            }
        ", None).unwrap();


        GliumRenderer {
            display: display,
            vertex_buffer: glium::VertexBuffer::new(display,
                vec![
                    Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] },
                    Vertex { position: [-1.0,  1.0], tex_coords: [0.0, 1.0] },
                    Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] },
                    Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 0.0] }
                ]
            ),
            index_buffer: glium::IndexBuffer::new(display,
                glium::index::TriangleStrip(vec![1 as u16, 2, 0, 3])),
            texture: glium::texture::CompressedTexture2d::new(&display, image),
            program: program,
        }
    }
}

impl<'a> uil::RenderContext for GliumRenderer<'a> {
    fn renderElement<B, R>(&mut self, boxi: &B, data: &R)
        where B: layout::Box, R: rendering::Material
    {
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            texture: &opengl_texture
        };

        // drawing a frame
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &std::default::Default::default()).unwrap();
        target.finish();
    }
}

fn main() {
    let display = glutin::WindowBuilder::new()
        .with_vsync()
        .build_glium()
        .unwrap();

    // TODO: FIXME load uil files

    // the main loop
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
