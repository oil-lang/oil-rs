use std::path::Path;
use glium::{self, Display};
use glium::texture::CompressedTexture2d;
use image::{self, GenericImage};
use oil_shared::resource::new_resource_id;


// Reexport

pub use oil_shared::resource::ResourceId;
pub use oil_shared::resource::BasicResourceManager;

// ======================================== //
//                INTERFACE                 //
// ======================================== //

pub trait ResourceManager: BasicResourceManager {

    fn get_texture(&self, id: ResourceId) -> &CompressedTexture2d;
}

pub fn create_resource_manager<'a>(display: &'a Display) -> ResourceManagerImpl {
    ResourceManagerImpl::new(display)
}

/// Create a ResourceManager that does nothing.
/// Usefull when you know that you won't have any resources
/// or that your program will stop after parsing.
pub fn create_null_manager() -> NullResourceManager {
    NullResourceManager
}

// ======================================== //
//                INTERNALS                 //
// ======================================== //

struct TextureResource {
    handle: glium::texture::CompressedTexture2d,
    img_width: u32,
    img_height: u32,
}

pub struct ResourceManagerImpl<'a> {
    textures: Vec<TextureResource>,
    display: &'a Display
}

impl<'a> ResourceManagerImpl<'a> {

    fn new(display: &'a Display) -> ResourceManagerImpl<'a> {
        ResourceManagerImpl {
            textures: Vec::new(),
            display: display,
        }
    }
}

impl<'a> BasicResourceManager for ResourceManagerImpl<'a> {

    fn get_texture_id(&mut self, p: &Path)
        -> ResourceId
    {
        let image = image::open(p).unwrap();
        let (iw, ih) = image.dimensions();
        let tex = glium::texture::CompressedTexture2d::new(self.display, image);
        let id = self.textures.len();
        self.textures.push(TextureResource {
            handle: tex,
            img_width: iw,
            img_height: ih
        });
        unsafe { new_resource_id(id) }
    }

    fn get_image_dimensions(
        &self,
        id: ResourceId)
        -> (u32, u32)
    {
        unsafe {
            (self.textures[id.get()].img_width, self.textures[id.get()].img_height)
        }
    }
}

impl<'a> ResourceManager for ResourceManagerImpl<'a> {

    fn get_texture(&self, id: ResourceId)
        -> &CompressedTexture2d
    {
        unsafe { &self.textures[id.get()].handle }
    }
}

pub struct NullResourceManager;

impl BasicResourceManager for NullResourceManager {

    fn get_texture_id(&mut self, _: &Path)
        -> ResourceId
    {
        unsafe { new_resource_id(0) }
    }

    fn get_image_dimensions(&self, _: ResourceId) -> (u32, u32) {
        (0, 0)
    }
}

impl ResourceManager for NullResourceManager {

    fn get_texture(&self, _: ResourceId)
        -> &CompressedTexture2d
    {
        panic!("NullResourceManager purpose is for test only,\
                it has a limited use.");
    }
}
