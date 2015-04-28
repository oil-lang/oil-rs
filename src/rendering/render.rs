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


// ======================================== //
//                   TESTS                  //
// ======================================== //

#[cfg(test)]
mod test {

    use super::RenderBuffer;
    use std::io::BufReader;
    use std::path::PathBuf;
    use glutin::WindowBuilder;
    use glium::DisplayBuild;
    use markup::{self, Node};
    use style::{self, Stylesheet};
    use uil_shared::deps::{Constructor, StyleDefinitions};
    use uil_parsers::{StdOutErrorReporter};
    use resource::{self, ResourceManager};

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

            let mut buffer = RenderBuffer::new(
                &fake_display,
                &fake_resource_manager,
                &style_tree
            );

            assert_eq!(buffer.render_data.len(), 2);
            let mut iter = buffer.render_data.enumerate_lookup_indices_mut().unwrap();
            let (&i, _) = iter.next().unwrap();
            assert_eq!(i, 1);
            let (&j, _) = iter.next().unwrap();
            assert_eq!(j, 3);
        }
    }
}
