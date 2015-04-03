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

    pub fn new<R>(
        display: &Display,
        resource_manager: &R,
        view: &markup::View,
        stylesheet: &style::Stylesheet)
        -> View
        where R: ResourceManager
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

    pub fn render<R, B>(
        &self,
        backend: &B,
        resource_manager: &R,
        frame: &mut <B as RenderBackbend>::Frame)
        where B: RenderBackbend,
              R: ResourceManager
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

fn create_render_buffer_and_lookup_table<R>(
    display: &Display,
    resource_manager: &R,
    style_tree: &StyledNode) -> (Box<[RenderData]>, Box<[usize]>)
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

    (buffer.into_boxed_slice(), lookup_table.into_boxed_slice())
}


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

// ======================================== //
//                   TESTS                  //
// ======================================== //

#[cfg(test)]
mod test {

    use std::io::BufReader;
    use std::path::PathBuf;
    use glutin::WindowBuilder;
    use glium::DisplayBuild;
    use markup::{self, Node};
    use style::{self, StyledNode, Stylesheet};
    use deps::{Constructor, StyleDefinitions};
    use super::create_render_buffer_and_lookup_table;
    use report;
    use resource::{self, ResourceManager};

    fn stylesheet<R: ResourceManager>(st: &str, r: &mut R) -> Stylesheet {
        let reader = BufReader::new(st.as_bytes());
        let mut defs = StyleDefinitions::new();
        defs.insert("toto".to_string(),
            Constructor::Image(PathBuf::new(), None, None, None, None));
        style::parse(report::StdOutErrorReporter, reader, &defs, r)
    }

    fn markup_tree(mk: &str) -> markup::Node {
        let reader = BufReader::new(mk.as_bytes());
        let lib = markup::parse(report::StdOutErrorReporter, reader);
        let (_, root) = lib.views.into_iter().next().unwrap();
        root
    }

    #[test]
    fn lookup_table_should_contain_correct_indices() {

        let mut fake_resource_manager = resource::create_stateless();
        let stylesheet = stylesheet(
            ".btn { background-image: $toto; }",
            &mut fake_resource_manager);
        let root = markup_tree(
            "<view>\
                <button class=\"btn\"></button>\
                <button class=\"\"></button>\
                <button class=\"btn\"></button>\
            </view>
            ");
        let style_tree = style::build_style_tree(&root, &stylesheet);
        let fake_display = WindowBuilder::new()
            .with_dimensions(1, 1).with_visibility(false).build_glium().unwrap();
        let (_, lookup_table) = create_render_buffer_and_lookup_table(
            &fake_display,
            &fake_resource_manager,
            &style_tree
        );

        assert_eq!(lookup_table[0], 1);
        assert_eq!(lookup_table[1], 3);
    }
}
