use std::ops::Deref;
use glium::Display;

use layout::LayoutBuffer;
use style::StyledNode;
use resource::ResourceManager;
use util::BufferFromTree;

use super::RenderData;
use super::TextureRule;

pub struct RenderBuffer {
    render_data: BufferFromTree<RenderData>,
}

impl Deref for RenderBuffer {
    type Target = [RenderData];

    fn deref<'a>(&'a self) -> &'a [RenderData] {
        self.render_data.deref()
    }
}

impl RenderBuffer {

    pub fn new<R>(
        display: &Display,
        resource_manager: &R,
        style_tree: &StyledNode) -> RenderBuffer
        where R: ResourceManager
    {
        // Create buffer with the magic number.
        // TODO: Stop magic, use real life example.
        let size = 10;

        let node_producer = |style_tree: &StyledNode| {
            if let Some(img) = style_tree.get_background_image() {
                let rule = style_tree.get_background_rule().unwrap_or(TextureRule::Fit);
                Some(RenderData::new(
                    display,
                    resource_manager,
                    img,
                    rule
                ))
            } else {
                None
            }
        };

        RenderBuffer {
            render_data: BufferFromTree::new_with_lookup_table(
                style_tree,
                size,
                node_producer
            )
        }
    }

    pub fn update_nodes(&mut self, display: &Display, layout_data: &LayoutBuffer) {

        for (&i, data) in self.render_data.enumerate_lookup_indices_mut().unwrap() {
            // This part is always safe because the initialization step
            // ensure that:
            //       self.layout_data.len() >= self.render_data
            let boxi = unsafe { layout_data.get_unchecked(i) };
            data.update_coords(display, &boxi);
        }
    }
}
