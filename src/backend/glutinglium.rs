use glium;
use image;

use std::default::Default;
use glium::{DisplayBuild, Surface, Display};
use image::{
    GenericImage,
    ImageBuffer
};

use RenderBackbend;
use layout;
use rendering;

#[derive(Copy)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

pub struct GliumRenderer<'a> {
    display: &'a Display,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer: glium::IndexBuffer,
    program: glium::Program,
}

impl<'a> GliumRenderer<'a> {
    pub fn new(display: &'a Display) -> GliumRenderer<'a> {

        let img = ImageBuffer::from_fn(1, 1, |_, _| {
            image::Luma([255u8])
        });

        let program = glium::Program::from_source(display, r"
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
                    Vertex { position: [-1.0, -1.0] },
                    Vertex { position: [-1.0,  1.0] },
                    Vertex { position: [ 1.0,  1.0] },
                    Vertex { position: [ 1.0, -1.0] }
                ]
            ),
            index_buffer: glium::IndexBuffer::new(display,
                glium::index::TriangleStrip(vec![1 as u16, 2, 0, 3])),
            program: program,
        }
    }
}

impl<'a> RenderBackbend for GliumRenderer<'a> {

    type Frame = glium::Frame;

    fn prepare_frame(&self) -> <GliumRenderer as RenderBackbend>::Frame {
        let mut f = self.display.draw();
        f.clear_color(0.0, 0.0, 0.0, 0.0);
        f
    }

    fn render_element(
        &self,
        frame: &mut <GliumRenderer as RenderBackbend>::Frame,
        boxi: &layout::LayoutBox,
        data: &rendering::RenderData)
    {
        match data.main_texture.as_ref() {
            Some(tex) => {

                let uniforms = uniform! {
                    matrix: [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0f32]
                    ],
                    texture: tex
                };

                frame.draw(
                    (&self.vertex_buffer, data.tex_coords_buffer.as_ref().unwrap()),
                    &self.index_buffer,
                    &self.program,
                    &uniforms,
                    &Default::default()).unwrap();
            }
            None => ()
        }
    }

    fn flush_frame(&self, frame: <GliumRenderer as RenderBackbend>::Frame) {
        frame.finish();
    }
}
