use std::slice;
use style::{StyledNode};

pub trait Material {
    fn texture(&self);
}

#[cfg(not(feature = "use_glium"))]
pub struct RenderData;

#[derive(Copy, Debug)]
pub enum TextureRule {
    Fit,
    Repeat
}

#[cfg(feature = "use_glium")]
pub use self::glutinglium::RenderData;
#[cfg(feature = "use_glium")]
mod glutinglium;


pub struct RenderBuffer(Box<[RenderData]>);

impl RenderBuffer {
    pub fn new(style_tree: &StyledNode) -> RenderBuffer {
        // Create buffer.
        let mut buffer = Vec::with_capacity(style_tree.tree_size());

        fill_buffer(&mut buffer, style_tree);

        RenderBuffer(buffer.into_boxed_slice())
    }

    pub fn iter(&self) -> slice::Iter<RenderData> {
        self.0.iter()
    }
}

impl Material for RenderData {
    fn texture(&self) {

    }
}

#[cfg(feature = "use_glium")]
fn fill_buffer(
    vec: &mut Vec<RenderData>,
    style_tree: &StyledNode)
{
    vec.push(
        RenderData::new(
            style_tree.get_background_image(),
            style_tree.get_background_rule()
        )
    );
    for kid in &style_tree.kids {
        fill_buffer(vec, kid);
    }
}

#[cfg(not(feature = "use_glium"))]
fn fill_buffer(
    vec: &mut Vec<RenderData>,
    style_tree: &StyledNode) {}
