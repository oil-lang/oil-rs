use glium::Display;

use resource::ResourceManager;
use layout::LayoutBuffer;
use focus::FocusBuffer;
use super::render::RenderBuffer;
use markup;
use RenderBackbend;
use Viewport;
use style;

pub struct View {
    dirty_flags: bool,
    focus_data: FocusBuffer,
    layout_data: LayoutBuffer,
    render_data: RenderBuffer,
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
        let focus_buffer = FocusBuffer::new(&stylenode);
        let render_buffer = RenderBuffer::new(display, resource_manager, &stylenode);

        View {
            dirty_flags: true,
            layout_data: layout_buffer,
            render_data: render_buffer,
            focus_data: focus_buffer,
        }
    }

    pub fn update(&mut self, display: &Display, vp: Viewport) {
        if self.dirty_flags {
            self.layout_data.compute_layout(vp.width, vp.height);
            self.render_data.update_nodes(display, &self.layout_data);
            self.focus_data.update_nodes(&self.layout_data);
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
    use style::{self, Stylesheet};
    use uil_shared::deps::{Constructor, StyleDefinitions};
    use uil_parsers::{StdOutErrorReporter};
    use resource::{self, ResourceManager};
    use super::create_render_buffer_and_lookup_table;

    fn stylesheet<R: ResourceManager>(st: &str, r: &mut R) -> Stylesheet {
        let reader = BufReader::new(st.as_bytes());
        let mut defs = StyleDefinitions::new();
        defs.insert("toto".to_string(),
            Constructor::Image(PathBuf::new(), None, None, None, None));
        style::parse(StdOutErrorReporter, reader, &defs, r)
    }

    fn markup_tree(mk: &str) -> markup::Node {
        let reader = BufReader::new(mk.as_bytes());
        let lib = markup::parse(StdOutErrorReporter, reader);
        let (_, root) = lib.views.into_iter().next().unwrap();
        root
    }

    #[test]
    fn lookup_table_should_contain_correct_indices() {

        let mut fake_resource_manager = resource::create_null_manager();
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

        // FIXME(This match is here to have the travis test pass)
        if let Some(fake_display) = WindowBuilder::new()
            .with_dimensions(1, 1).with_visibility(false).build_glium().ok()
        {

            let (_, lookup_table) = create_render_buffer_and_lookup_table(
                &fake_display,
                &fake_resource_manager,
                &style_tree
            );

            assert_eq!(lookup_table[0], 1);
            assert_eq!(lookup_table[1], 3);
        }
    }
}
