
use layout::LayoutBuffer;
use rendering::RenderBuffer;
use markup;
use RenderContext;
use style;

pub struct View {
    layout_data: LayoutBuffer,
    render_data: RenderBuffer,
}

impl View {

    pub fn new(view: &markup::View, stylesheet: &style::Stylesheet) -> View
    {
        let stylenode = style::build_style_tree(view, stylesheet);
        let layout_buffer = LayoutBuffer::new(&stylenode);
        View {
            layout_data: layout_buffer,
            render_data: RenderBuffer,
        }
    }

    pub fn render<C>(&self, ctx: &mut C)
        where C: RenderContext
    {
        for (boxi, data) in self.layout_data.iter().zip(self.render_data.iter()) {
            ctx.render_element(boxi, &data);
        }
    }
}
