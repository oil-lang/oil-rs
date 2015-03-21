
use layout::LayoutBuffer;
use rendering::RenderBuffer;
use RenderContext;

pub struct View {
    layout_data: LayoutBuffer,
    render_data: RenderBuffer,
}

impl View {
    pub fn render<C>(&self, ctx: &mut C)
        where C: RenderContext
    {
        for (boxi, data) in self.layout_data.iter().zip(self.render_data.iter()) {
            ctx.render_element(boxi, &data);
        }
    }
}
