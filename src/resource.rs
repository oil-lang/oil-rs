use std::path::Path;
use glium::{self, Display};
use image::{self, GenericImage};

#[derive(Copy, Clone)]
pub struct ResourceId(usize);

struct TextureResource {
    handle: glium::texture::CompressedTexture2d,
    img_width: u32,
    img_height: u32,
}

pub struct ResourceManager<'a> {
    textures: Vec<TextureResource>,
    display: &'a Display
}

impl<'a> ResourceManager<'a> {

    pub fn new(display: &'a Display) -> ResourceManager<'a> {
        ResourceManager {
            textures: Vec::new(),
            display: display,
        }
    }

    pub fn get_texture_id(&mut self, p: &Path) -> ResourceId {
        let image = image::open(p).unwrap();
        let (iw, ih) = image.dimensions();
        let tex = glium::texture::CompressedTexture2d::new(self.display, image);
        let id = self.textures.len();
        self.textures.push(TextureResource {
            handle: tex,
            img_width: iw,
            img_height: ih
        });
        ResourceId(id)
    }

    pub fn get_texture(&self, id: ResourceId) -> &glium::texture::CompressedTexture2d {
        &self.textures[id.0].handle
    }

    pub fn get_image_dimensions(&self, id: ResourceId) -> (u32, u32) {
        (self.textures[id.0].img_width, self.textures[id.0].img_height)
    }
}
