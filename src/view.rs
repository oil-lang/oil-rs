
use layout::LayoutBuffer;
use rendering::RenderBuffer;
use glium::Display;

use resource::ResourceManager;
use markup;
use RenderBackbend;
use Viewport;
use style;

pub struct View {
    dirty_flags: bool,
    layout_data: LayoutBuffer,
    render_data: RenderBuffer,
}

impl View {

    pub fn new(display: &Display, resource_manager: &ResourceManager, view: &markup::View, stylesheet: &style::Stylesheet)
        -> View
    {
        let stylenode = style::build_style_tree(view, stylesheet);
        let layout_buffer = LayoutBuffer::new(&stylenode);
        let render_buffer = RenderBuffer::new(display, resource_manager, &stylenode);
        View {
            dirty_flags: true,
            layout_data: layout_buffer,
            render_data: render_buffer,
        }
    }

    pub fn update(&mut self, display: &Display, vp: Viewport) {
        if self.dirty_flags {
            self.layout_data.compute_layout(vp.width, vp.height);
            self.render_data.update_buffers(display, &self.layout_data);
            self.dirty_flags = false;
        }
    }

    pub fn render<B>(&self, backend: &B, resource_manager: &ResourceManager, frame: &mut <B as RenderBackbend>::Frame)
        where B: RenderBackbend
    {
        for data in self.render_data.iter() {
            backend.render_element(resource_manager, frame, data);
        }
    }
}
