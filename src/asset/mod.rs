use std::num::ToPrimitive;
use std::rc::Rc;
use std::fmt::{self, Debug};
use image::{self, GenericImage};

use deps::Constructor;

#[derive(Debug, Clone)]
pub struct FontData;

// TODO handle shared images somehow
// even in a disgusting way, but something !
// The opengl backend will do it by using the same texture id.
#[derive(Clone)]
pub struct ImageData {
    img: Rc<image::DynamicImage>,
    offset_x: f32,
    offset_y: f32,
    width: f32,
    height: f32,
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

    pub fn new(imageCtor: &Constructor) -> ImageData {
        if let Constructor::Image(ref path, width, height, offset_x, offset_y)
                = *imageCtor
        {
            let image = image::open(&Path::new(path)).unwrap();
            let (iw, ih) = image.dimensions();
            let w = width.unwrap_or(iw.to_f32().unwrap());
            let h = height.unwrap_or(ih.to_f32().unwrap());
            let x = offset_x.unwrap_or(0f32);
            let y = offset_y.unwrap_or(0f32);

            ImageData {
                img: Rc::new(image),
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

    pub fn new(fontCtor: &Constructor) -> FontData {
        if let Constructor::Font(ref path, width, height) = *fontCtor {
            // TODO: see freetype-rs or something similar
            FontData
        } else {
            panic!("Wrong constructor passed. Expected Constructor::Font.");
        }
    }
}
