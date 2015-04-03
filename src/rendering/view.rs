use std::slice;
use style::{StyledNode};
use glium::Display;

use resource::ResourceManager;
use layout::LayoutBuffer;
use markup;
use RenderBackbend;
use Viewport;
use style;

use super::RenderData;
use super::TextureRule;

pub struct View {
    dirty_flags: bool,
    layout_data: LayoutBuffer,
    render_data: Box<[RenderData]>,
    lookup_indices: Box<[usize]>,
}

impl View {

    pub fn new(
        display: &Display,
        resource_manager: &ResourceManager,
        view: &markup::View,
        stylesheet: &style::Stylesheet)
        -> View
    {
        let stylenode = style::build_style_tree(view, stylesheet);
        let layout_buffer = LayoutBuffer::new(&stylenode);
        let (render_buffer, lookup_table) = create_render_buffer_and_lookup_table(
            display,
            resource_manager,
            &stylenode
        );
        View {
            dirty_flags: true,
            layout_data: layout_buffer,
            render_data: render_buffer,
            lookup_indices: lookup_table,
        }
    }

    pub fn update(&mut self, display: &Display, vp: Viewport) {
        if self.dirty_flags {
            self.layout_data.compute_layout(vp.width, vp.height);
            self.update_buffers(display);
            self.dirty_flags = false;
        }
    }

    pub fn render<B>(
        &self,
        backend: &B,
        resource_manager: &ResourceManager,
        frame: &mut <B as RenderBackbend>::Frame)
        where B: RenderBackbend
    {
        for data in self.render_data.iter() {
            backend.render_element(resource_manager, frame, data);
        }
    }

    fn update_buffers(&mut self, display: &Display) {
        let ref layout_data = self.layout_data;
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

fn create_render_buffer_and_lookup_table(
    display: &Display,
    resource_manager: &ResourceManager,
    style_tree: &StyledNode) -> (Box<[RenderData]>, Box<[usize]>)
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
        0,
        style_tree
    );

    (buffer.into_boxed_slice(), lookup_table.into_boxed_slice())
}


fn fill_buffer(
    display: &Display,
    resource_manager: &ResourceManager,
    buffer: &mut Vec<RenderData>,
    lookup_table: &mut Vec<usize>,
    mut layout_box_ref: usize,
    style_tree: &StyledNode)
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
            layout_box_ref
        );

    }

    for kid in &style_tree.kids {
        layout_box_ref += 1;
        fill_buffer(display, resource_manager, buffer, lookup_table,
            layout_box_ref, kid);
    }
}
