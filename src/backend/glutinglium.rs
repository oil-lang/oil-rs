extern crate glutin;
extern crate glium;
extern crate image;

use std::default::Default;
use glium::{DisplayBuild, Surface, Display};

use RenderContext;
use layout;
use rendering;

#[derive(Copy)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

pub struct GliumRenderer<'a> {
    display: &'a Display,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer: glium::IndexBuffer,
    texture: glium::texture::CompressedTexture2d,
    program: glium::Program,
}

impl<'a> GliumRenderer<'a> {
    pub fn new(display: &'a Display, img: image::DynamicImage) -> GliumRenderer<'a> {

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
            texture: glium::texture::CompressedTexture2d::new(&display, img),
            program: program,
        }
    }
}

impl<'a> RenderContext for GliumRenderer<'a> {
    fn render_element<B, R>(&mut self, boxi: &B, data: &R)
        where B: layout::Box, R: rendering::Material
    {
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            texture: &self.texture
        };

        // drawing a frame
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(
            &self.vertex_buffer,
            &self.index_buffer,
            &self.program,
            &uniforms,
            &Default::default()).unwrap();
        target.finish();
    }
}
