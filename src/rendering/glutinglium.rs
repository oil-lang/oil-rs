use glium;
use asset::ImageData;
use super::TextureRule;

pub struct RenderData {
    main_texure: Option<glium::texture::CompressedTexture2d>,
}

impl RenderData {

    pub fn new(img: Option<ImageData>, rule: Option<TextureRule>) -> RenderData {
        // TODO: FIXME
        RenderData {
            main_texure: None
        }
    }
}
