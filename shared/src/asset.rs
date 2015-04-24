use std::fmt::{self, Debug};
use std::path::Path;

use deps::Constructor;
use resource::{BasicResourceManager, ResourceId};

#[derive(Debug, Clone)]
pub struct FontData;

// TODO handle shared images somehow
// even in a disgusting way, but something !
// The opengl backend will do it by using the same texture id.
#[derive(Clone)]
pub struct ImageData {
    pub img: ResourceId,
    pub offset_x: f32,
    pub offset_y: f32,
    pub width: f32,
    pub height: f32,
}

/// Necessary because DynamicImage does not implement the trait Debug.
impl Debug for ImageData
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "ImageData {{ "));

        try!(write!(f, "offset_x {:?}, ", self.offset_x));
        try!(write!(f, "offset_y {:?}, ", self.offset_y));
        try!(write!(f, "width {:?}, ", self.width));
        try!(write!(f, "height {:?} ", self.height));

        write!(f, "}}")
    }
}
impl ImageData {

    pub fn new<R>(
        image_ctor: &Constructor,
        resource_manager: &mut R)
        -> ImageData
        where R: BasicResourceManager
    {
        if let Constructor::Image(ref path, width, height, offset_x, offset_y)
                = *image_ctor
        {
            let image = resource_manager.get_texture_id(&Path::new(path));
            let (iw, ih) = resource_manager.get_image_dimensions(image);
            let w = width.unwrap_or(iw as f32);
            let h = height.unwrap_or(ih as f32);
            let x = offset_x.unwrap_or(0f32);
            let y = offset_y.unwrap_or(0f32);

            ImageData {
                img: image,
                offset_x: x,
                offset_y: y,
                width: w,
                height: h,
            }
        } else {
            panic!("Wrong constructor passed. Expected Constructor::Image.");
        }
    }
}

impl FontData {

    pub fn new(font_ctor: &Constructor) -> FontData {
        if let Constructor::Font(ref path, width, height) = *font_ctor {
            // TODO: see freetype-rs or something similar
            FontData
        } else {
            panic!("Wrong constructor passed. Expected Constructor::Font.");
        }
    }
}
