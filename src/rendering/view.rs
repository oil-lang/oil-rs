use glium::Display;

use resource::ResourceManager;
use layout::LayoutBuffer;
use state::StateBuffer;
use focus::{self, FocusBuffer};
use super::render::RenderBuffer;
use uil_shared::style::Stylesheet;
use markup;
use RenderBackbend;
use Viewport;

pub struct View {
    dirty_flags: bool,
    state_data: StateBuffer,
    focus_data: FocusBuffer,
    focus_node: isize,
    layout_data: LayoutBuffer,
    render_data: RenderBuffer,
}

impl View {

    pub fn new<R>(
        display: &Display,
        resource_manager: &R,
        view: &markup::View,
        stylesheet: &Stylesheet)
        -> View
        where R: ResourceManager
    {
        let state_buffer = StateBuffer::new(view, stylesheet);
        let focus_buffer = FocusBuffer::new(view);
        let layout_buffer = LayoutBuffer::new(view);
        let render_buffer = RenderBuffer::new(display, resource_manager, &state_buffer);

        View {
            dirty_flags: true,
            layout_data: layout_buffer,
            render_data: render_buffer,
            focus_node: focus_buffer.first_acceptor_index(),
            focus_data: focus_buffer,
            state_data: state_buffer,
        }
    }

    pub fn update(&mut self, display: &Display, vp: Viewport) {
        if self.dirty_flags {
            self.layout_data.update_from_state(&self.state_data);
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

    pub fn focus_up(&mut self) {
        if self.focus_node >= 0 {
            assert!((self.focus_node as usize) < self.focus_data.len());

            self.focus_node = self.focus_data.node_as_index(
                focus::focus_up(&self.focus_data[self.focus_node])
            );
        }
    }

    pub fn focus_down(&mut self) {
        if self.focus_node >= 0 {
            assert!((self.focus_node as usize) < self.focus_data.len());

            self.focus_node = self.focus_data.node_as_index(
                focus::focus_down(&self.focus_data[self.focus_node])
            );
        }
    }

    pub fn focus_right(&mut self) {
        if self.focus_node >= 0 {
            assert!((self.focus_node as usize) < self.focus_data.len());

            self.focus_node = self.focus_data.node_as_index(
                focus::focus_right(&self.focus_data[self.focus_node])
            );
        }
    }

    pub fn focus_left(&mut self) {
        if self.focus_node >= 0 {
            assert!((self.focus_node as usize) < self.focus_data.len());

            self.focus_node = self.focus_data.node_as_index(
                focus::focus_left(&self.focus_data[self.focus_node])
            );
        }
    }
}


// ======================================== //
//                   TESTS                  //
// ======================================== //
