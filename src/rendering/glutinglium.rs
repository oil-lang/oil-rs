use std::num::ToPrimitive;
use glium;
use glium::Display;
use image::{self, GenericImage};

use asset::ImageData;
use layout::LayoutBox;
use resource::{ResourceManager, ResourceId};
use super::TextureRule;

/// TODO: Remove this public qualifier somehow.
#[derive(Copy)]
pub struct TexCoords {
    tex_coords: [f32; 2],
}

#[derive(Copy)]
pub struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);
implement_vertex!(TexCoords, tex_coords);

pub struct RenderData {
    pub main_texture: Option<ResourceId>,
    pub tex_coords_buffer: Option<glium::VertexBuffer<TexCoords>>,
    pub vertex_coords_buffer: Option<glium::VertexBuffer<Vertex>>,
}

impl RenderData {

    pub fn new(
        display: &Display,
        resource_manager: &ResourceManager,
        img: Option<ImageData>,
        rule: Option<TextureRule>)
        -> RenderData
    {

        // If we have an image then we load the rendering details.
        // TODO: clean up with the resource manager
        if let Some(image) = img {

            let (iw, ih) = resource_manager.get_image_dimensions(image.img);
            let (w_m, h_m) = (iw.to_f32().unwrap(), ih.to_f32().unwrap());
            let x = image.offset_x / w_m;
            let y = image.offset_y / h_m;
            let xo = image.width  / w_m + x;
            let yo = image.height / h_m + y;

            let buffer = glium::VertexBuffer::new(display,
                vec![
                    TexCoords { tex_coords: [ x, y ] },
                    TexCoords { tex_coords: [ x, yo] },
                    TexCoords { tex_coords: [xo, yo] },
                    TexCoords { tex_coords: [xo, y ] }
                ]
            );

            RenderData {
                main_texture: Some(image.img),
                tex_coords_buffer: Some(buffer),
                vertex_coords_buffer: None,
            }
        } else {
            RenderData {
                main_texture: None,
                tex_coords_buffer: None,
                vertex_coords_buffer: None,
            }
        }
    }

    pub fn update_coords(&mut self, display: &Display, lb: &LayoutBox) {
        // TODO: Look how to do a glMapBuffer instead of this when
        // vertex_coords_buffer is Some(buffer).
        // Note: for now  it should be acceptable as this is probably
        //       not the bottle neck.
        if self.main_texture.is_some() {

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
}
