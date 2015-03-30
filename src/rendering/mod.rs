use std::slice;
use style::{StyledNode};
use glium::Display;

use resource::ResourceManager;

#[derive(Copy, Debug)]
pub enum TextureRule {
    Fit,
    Repeat
}

pub use self::glutinglium::RenderData;

mod glutinglium;

pub struct RenderBuffer(Box<[RenderData]>);

impl RenderBuffer {
    pub fn new(display: &Display, resource_manager: &ResourceManager, style_tree: &StyledNode) -> RenderBuffer {
        // Create buffer.
        let mut buffer = Vec::with_capacity(style_tree.tree_size());

        fill_buffer(display, resource_manager, &mut buffer, style_tree);

        RenderBuffer(buffer.into_boxed_slice())
    }

    pub fn iter(&self) -> slice::Iter<RenderData> {
        self.0.iter()
    }
}

fn fill_buffer(
    display: &Display,
    resource_manager: &ResourceManager,
    vec: &mut Vec<RenderData>,
    style_tree: &StyledNode)
{
    vec.push(
        RenderData::new(
            display,
            resource_manager,
            style_tree.get_background_image(),
            style_tree.get_background_rule()
        )
    );
    for kid in &style_tree.kids {
        fill_buffer(display, resource_manager, vec, kid);
    }
}
