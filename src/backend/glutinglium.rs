use glium;
use cgmath;

use std::default::Default;
use glium::{Surface, Display};
use image::{
    GenericImage
};

use resource::ResourceManager;
use Viewport;
use RenderBackbend;
use rendering;

pub struct GliumRenderer<'a> {
    display: &'a Display,
    index_buffer: glium::IndexBuffer,
    program: glium::Program,
    matrix: cgmath::Matrix4<f32>,
}

impl<'a> GliumRenderer<'a> {
    pub fn new(display: &'a Display) -> GliumRenderer<'a> {

        let program = glium::Program::from_source(display, r"
            #version 110

            uniform mat4 matrix;

            attribute vec2 position;
            attribute vec2 tex_coords;

            varying vec2 v_tex_coords;

            void main() {
                gl_Position = matrix * vec4(position, 0.0, 1.0);
                v_tex_coords = vec2(tex_coords.x, 1.0 - tex_coords.y);
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
            index_buffer: glium::IndexBuffer::new(display,
                glium::index::TriangleStrip(vec![1u32, 2, 0, 3])),
            program: program,
            matrix: cgmath::Matrix4::zero()
        }
    }
}

impl<'a> RenderBackbend for GliumRenderer<'a> {

    type Frame = glium::Frame;

    fn prepare_frame(&mut self, vp: Viewport) -> <GliumRenderer as RenderBackbend>::Frame {
        let mut f = self.display.draw();
        self.matrix = cgmath::ortho(0.0, vp.width, vp.height, 0.0, 0.0, 1.0);
        f.clear_color(0.0, 0.0, 0.0, 0.0);
        f
    }

    fn render_element(
        &self,
        resource_manager: &ResourceManager,
        frame: &mut <GliumRenderer as RenderBackbend>::Frame,
        data: &rendering::RenderData)
    {
        match data.main_texture {
            Some(tex_id) => {

                let tex = resource_manager.get_texture(tex_id);
                let uniforms = uniform! {
                    matrix: self.matrix,
                    texture: tex
                };

                let vb = data.vertex_coords_buffer.as_ref().unwrap();
                let tb = data.tex_coords_buffer.as_ref().unwrap();

                frame.draw(
                    (vb, tb),
                    &self.index_buffer,
                    &self.program,
                    &uniforms,
                    &Default::default()).unwrap();
            }
            _ => ()
        }
    }

    fn flush_frame(&self, frame: <GliumRenderer as RenderBackbend>::Frame) {
        frame.finish();
    }
}
