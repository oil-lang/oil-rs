use std::num::ToPrimitive;
use glium;
use glium::Display;
use image::{self, GenericImage};

use asset::ImageData;
use super::TextureRule;

/// TODO: Remove this public qualifier somehow.
#[derive(Copy)]
pub struct TexCoords {
    tex_coords: [f32; 2],
}

implement_vertex!(TexCoords, tex_coords);

pub struct RenderData {
    pub main_texture: Option<glium::texture::CompressedTexture2d>,
    pub tex_coords_buffer: Option<glium::VertexBuffer<TexCoords>>,
}

impl RenderData {

    pub fn new(display: &Display, img: Option<ImageData>, rule: Option<TextureRule>) -> RenderData {

        // If we have an image then we load the rendering details.
        // TODO: clean up with the resource manager
        if let Some(image) = img {

            let (iw, ih) = image.img().dimensions();
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

            let tex = glium::texture::CompressedTexture2d::new(display, *image.img());

            RenderData {
                main_texture: Some(tex),
                tex_coords_buffer: Some(buffer),
            }
        } else {
            RenderData {
                main_texture: None,
                tex_coords_buffer: None,
            }
        }
    }
}
