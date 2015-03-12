use std::iter;

pub trait Material {
    fn texture(&self);
}


pub struct RenderBuffer;
#[derive(Clone)]
pub struct RenderData;

impl RenderBuffer {
    pub fn new() -> RenderBuffer {
        RenderBuffer
    }

    pub fn iter(&self) -> iter::Repeat<RenderData> {
        iter::repeat(RenderData)
    }
}

impl Material for RenderData {
    fn texture(&self) {

    }
}
