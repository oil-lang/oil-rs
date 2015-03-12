
use layout::LayoutBuffer;
use rendering::RenderBuffer;
use RenderContext;

pub struct View {
    layoutData: LayoutBuffer,
    renderData: RenderBuffer,
}

impl View {
    pub fn render<C>(&self, ctx: &mut C)
        where C: RenderContext
    {
        for (boxi, data) in self.layoutData.iter().zip(self.renderData.iter()) {
            ctx.renderElement(boxi, &data);
        }
    }
}
