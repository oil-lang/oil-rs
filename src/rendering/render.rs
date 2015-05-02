use std::ops::Deref;
use glium::Display;

use layout::LayoutBuffer;
use resource::ResourceManager;
use util::BufferFromTree;
use state::StateBuffer;
use state::StateData;

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
        state_buffer: &StateBuffer) -> RenderBuffer
        where R: ResourceManager
    {
        let node_producer = |state: &StateData| {
            if let Some(img) = state.get_background_image() {
                let rule = state.get_background_rule().unwrap_or(TextureRule::Fit);
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
            render_data: BufferFromTree::from_buffer(
                state_buffer,
                node_producer
            )
        }
    }

    pub fn update_from_state<R: ResourceManager>(
        &mut self,
        display: &Display,
        resource_manager: &R,
        state_data: &StateBuffer)
    {

        for (&i, data) in self.render_data.enumerate_lookup_indices_mut().unwrap() {

            let state = unsafe { state_data.get_unchecked(i) };

            if let Some(img) = state.get_background_image() {

                data.update_texture(display, resource_manager, img);
            }
        }
    }

    pub fn update_from_layout(&mut self, display: &Display, layout_data: &LayoutBuffer) {

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
    use style;
    use state::StateBuffer;
    use uil_shared::style::Stylesheet;
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
        let state_buffer = StateBuffer::new(&root, &stylesheet);

        // FIXME(This match is here to have the travis test pass)
        if let Some(fake_display) = WindowBuilder::new()
            .with_dimensions(1, 1).with_visibility(false).build_glium().ok()
        {

            let mut buffer = RenderBuffer::new(
                &fake_display,
                &fake_resource_manager,
                &state_buffer
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
