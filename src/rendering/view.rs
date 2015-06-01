use glium::Display;

use resource::ResourceManager;
use layout::LayoutBuffer;
use state::StateBuffer;
use focus::{FocusBuffer, FocusedElement};
use super::render::RenderBuffer;
use oil_shared::style::SelectorState;
use oil_shared::style::Stylesheet;
use data_bindings::{DataBindingBuffer,DataBinderContext};
use markup;
use RenderBackbend;
use Viewport;

pub struct View {
    dirty_flags: bool,
    // Buffers
    state_data: StateBuffer,
    focus_data: FocusBuffer,
    layout_data: LayoutBuffer,
    render_data: RenderBuffer,
    // Current state
    current_focused_node: FocusedElement,
    data_binding_buffer: DataBindingBuffer,
}

impl View {

    pub fn new<R>(
        display: &Display,
        resource_manager: &R,
        view: &markup::View,
        templates: &HashMap<String, markup::Template>,
        stylesheet: &Stylesheet)
        -> View
        where R: ResourceManager
    {
        let state_buffer = StateBuffer::new(view, stylesheet);
        let focus_buffer = FocusBuffer::new(view);
        let layout_buffer = LayoutBuffer::new(view);
        let render_buffer = RenderBuffer::new(display, resource_manager, &state_buffer);
        let data_binding_buffer = DataBindingBuffer::new(view, templates);

        View {
            dirty_flags: true,
            layout_data: layout_buffer,
            render_data: render_buffer,
            current_focused_node: focus_buffer.first_acceptor(),
            focus_data: focus_buffer,
            state_data: state_buffer,
            data_binding_buffer: data_binding_buffer,
        }
    }

    pub fn update<R>(
        &mut self,
        display: &Display,
        resource_manager: &R,
        vp: Viewport,
        context: &DataBinderContext)
        where R: ResourceManager
    {
        let updated_bindings = self.data_binding_buffer.update(context, &mut self.layout_data);
        if self.dirty_flags || updated_bindings {
            self.set_state_for_focused_node();
            self.layout_data.update_from_state(&self.state_data);
            self.layout_data.compute_layout(vp.width, vp.height);
            self.render_data.update_from_state(display, resource_manager, &self.state_data);
            self.render_data.update_from_layout(display, &self.layout_data);
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
        if let Some(new_focused_node) = self.focus_data.focus_up(&self.current_focused_node) {
            self.remove_state_for_focused_node();
            self.current_focused_node = new_focused_node;
            self.dirty_flags = true;
        }
    }

    pub fn focus_down(&mut self) {
        if let Some(new_focused_node) = self.focus_data.focus_down(&self.current_focused_node) {
            self.remove_state_for_focused_node();
            self.current_focused_node = new_focused_node;
            self.dirty_flags = true;
        }
    }

    pub fn focus_right(&mut self) {
        if let Some(new_focused_node) = self.focus_data.focus_right(&self.current_focused_node) {
            self.remove_state_for_focused_node();
            self.current_focused_node = new_focused_node;
            self.dirty_flags = true;
        }
    }

    pub fn focus_left(&mut self) {
        if let Some(new_focused_node) = self.focus_data.focus_left(&self.current_focused_node) {
            self.remove_state_for_focused_node();
            self.current_focused_node = new_focused_node;
            self.dirty_flags = true;
        }
    }

    fn set_state_for_focused_node(&mut self) {

        if let Some(global_index) = self.focus_data.global_index(&self.current_focused_node) {

            self.state_data.get_mut(global_index)
                .unwrap()
                .set_current_state(SelectorState::Focus);
        }
    }

    fn remove_state_for_focused_node(&mut self) {

        if let Some(global_index) = self.focus_data.global_index(&self.current_focused_node) {

            self.state_data.get_mut(global_index)
                .unwrap()
                .set_current_state(SelectorState::Default);
        }
    }
}


// ======================================== //
//                   TESTS                  //
// ======================================== //
