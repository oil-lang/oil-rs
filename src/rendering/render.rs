use std::ops::Deref;
use glium::Display;

use layout::LayoutBuffer;
use style::StyledNode;
use resource::ResourceManager;

use super::RenderData;
use super::TextureRule;

pub struct RenderBuffer {
    render_data: Box<[RenderData]>,
    lookup_indices: Box<[usize]>,
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
        let mut buffer = Vec::with_capacity(10);
        let mut lookup_table = Vec::with_capacity(10);

        fill_buffer(
            display,
            resource_manager,
            &mut buffer,
            &mut lookup_table,
            &mut 0,
            style_tree
        );

        RenderBuffer {
            render_data: buffer.into_boxed_slice(),
            lookup_indices: lookup_table.into_boxed_slice(),
        }
    }

    pub fn update_nodes(&mut self, display: &Display, layout_data: &LayoutBuffer) {

        for (data, &i) in self.render_data.iter_mut().zip(self.lookup_indices.iter()) {
            // This part is always safe because the initialization step
            // ensure that:
            //       self.layout_data.len() >= self.render_data
            let boxi = unsafe { layout_data.get_unchecked(i) };
            data.update_coords(display, &boxi);
        }
    }
}

// ======================================== //
//                  HELPERS                 //
// ======================================== //

fn fill_buffer<R>(
    display: &Display,
    resource_manager: &R,
    buffer: &mut Vec<RenderData>,
    lookup_table: &mut Vec<usize>,
    layout_box_ref: &mut usize,
    style_tree: &StyledNode)
    where R: ResourceManager
{
    if let Some(img) = style_tree.get_background_image() {

        let rule = style_tree.get_background_rule().unwrap_or(TextureRule::Fit);

        buffer.push(
            RenderData::new(
                display,
                resource_manager,
                img,
                rule
            )
        );

        lookup_table.push(
            *layout_box_ref
        );

    }
    *layout_box_ref += 1;

    for kid in &style_tree.kids {
        fill_buffer(display, resource_manager, buffer, lookup_table,
            layout_box_ref, kid);
    }
}
