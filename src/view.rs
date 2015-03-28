
use layout::LayoutBuffer;
use rendering::RenderBuffer;
use glium::Display;

use markup;
use RenderBackbend;
use Viewport;
use style;

pub struct View {
    dirty_flags: DirtyViewFlags,
    layout_data: LayoutBuffer,
    render_data: RenderBuffer,
}

impl View {

    pub fn new(display: &Display, view: &markup::View, stylesheet: &style::Stylesheet)
        -> View
    {
        let stylenode = style::build_style_tree(view, stylesheet);
        let layout_buffer = LayoutBuffer::new(&stylenode);
        let render_buffer = RenderBuffer::new(display, &stylenode);
        View {
            dirty_flags: LAYOUT_IS_DIRTY | RENDER_IS_DIRTY,
            layout_data: layout_buffer,
            render_data: render_buffer,
        }
    }

    pub fn update(&mut self, vp: Viewport) {
        if self.dirty_flags.contains(LAYOUT_IS_DIRTY) {
            self.layout_data.compute_layout(vp.width, vp.height);
            self.dirty_flags.remove(LAYOUT_IS_DIRTY);
        }
    }

    pub fn render<B>(&self, backend: &mut B, frame: &mut <B as RenderBackbend>::Frame)
        where B: RenderBackbend
    {
        for (boxi, data) in self.layout_data.iter().zip(self.render_data.iter()) {
            backend.render_element(frame, boxi, data);
        }
    }
}

bitflags! {
    flags DirtyViewFlags: u8 {
        const LAYOUT_IS_DIRTY = 0b01,
        const RENDER_IS_DIRTY = 0b10,
    }
}
