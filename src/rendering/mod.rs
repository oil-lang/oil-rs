// ======================================== //
//                INTERFACE                 //
// ======================================== //

pub mod backend;
pub use self::view::View;

mod view;
mod render;

#[derive(Copy, Clone, Debug)]
pub enum TextureRule {
    Fit,
    Repeat
}

// ======================================== //
//                INTERNALS                 //
// ======================================== //

use num::traits::ToPrimitive;
use glium;
use glium::Display;
use image::{GenericImage};

use oil_shared::asset::ImageData;
use layout::LayoutBox;
use resource::{ResourceManager, ResourceId};

#[derive(Copy, Clone)]
struct TexCoords {
    tex_coords: [f32; 2],
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);
implement_vertex!(TexCoords, tex_coords);

pub struct RenderData {
    main_texture: ResourceId,
    tex_coords_buffer: glium::VertexBuffer<TexCoords>,
    vertex_coords_buffer: Option<glium::VertexBuffer<Vertex>>,
    rule: TextureRule,
}

impl RenderData {

    fn new<R: ResourceManager>(
        display: &Display,
        resource_manager: &R,
        image: ImageData,
        rule: TextureRule)
        -> RenderData
    {

        let buffer = RenderData::initialize_buffer(display, resource_manager, &image);

        RenderData {
            main_texture: image.img,
            tex_coords_buffer: buffer,
            vertex_coords_buffer: None,
            rule: rule,
        }
    }

    fn initialize_buffer<R: ResourceManager>(
        display: &Display,
        resource_manager: &R,
        image: &ImageData)
        -> glium::VertexBuffer<TexCoords>
    {
        // TODO: Handle TextureRule::Repeat
        let (iw, ih) = resource_manager.get_image_dimensions(image.img);
        let (w_m, h_m) = (iw.to_f32().unwrap(), ih.to_f32().unwrap());
        let x = image.offset_x / w_m;
        let y = image.offset_y / h_m;
        let xo = image.width  / w_m + x;
        let yo = image.height / h_m + y;

        glium::VertexBuffer::new(display,
            vec![
                TexCoords { tex_coords: [ x, y ] },
                TexCoords { tex_coords: [ x, yo] },
                TexCoords { tex_coords: [xo, yo] },
                TexCoords { tex_coords: [xo, y ] }
            ]
        )
    }

    fn update_texture<R: ResourceManager>(
        &mut self,
        display: &Display,
        rm: &R,
        image: ImageData)
    {

        // TODO: Update existing buffer instead
        self.tex_coords_buffer = RenderData::initialize_buffer(display, rm, &image);
        self.main_texture = image.img;
    }

    fn update_coords(&mut self, display: &Display, lb: &LayoutBox) {
        // TODO: Look how to do a glMapBuffer instead of this when
        // vertex_coords_buffer is Some(buffer).
        // Note: for now  it should be acceptable as this is probably
        //       not the bottle neck.
        let x = lb.dim().content.x + lb.dim().margin.left;
        let y = lb.dim().content.y + lb.dim().margin.top;
        let height = lb.dim().content.height;
        let width = lb.dim().content.width;

        self.vertex_coords_buffer = Some(
            glium::VertexBuffer::new(display, vec![
                Vertex { position: [         x, y         ]},
                Vertex { position: [         x, y + height]},
                Vertex { position: [ x + width, y + height]},
                Vertex { position: [ x + width, y         ]}
            ])
        );
    }
}
