#![feature(plugin)]
#![feature(iter_cmp)]
#![plugin(phf_macros)]

#[macro_use]
extern crate bitflags;
extern crate phf;
extern crate num;

#[macro_use]
extern crate glium;
extern crate image;
extern crate cgmath;
extern crate oil_parsers;
extern crate oil_shared;
extern crate oil_databindings;

#[cfg(test)]
extern crate glutin;

pub mod markup;
pub mod style;
pub mod deps;
pub mod rendering;
pub mod resource;
pub mod data_bindings;

// Reexport
pub use oil_parsers::ErrorReporter;
pub use oil_parsers::StdOutErrorReporter;
pub use oil_parsers::EmptyErrorReporter;
pub use self::router::Router;
pub use self::rendering::View;
pub use self::data_bindings::DefaultContextManager;
pub use self::data_bindings::Store;
pub use self::data_bindings::DataBindingsContext;

mod layout;
mod router;
mod util;
mod focus;
mod state;

/// Trait used by oil to perform the high level rendering operations.
/// Ideally, `oil` should not depend on a specific implementation.
///
/// It turns out, today's this is not true at all.
pub trait RenderBackbend {
    type Frame;

    /// Prepare the frame for the current rendering step.
    fn prepare_frame(&mut self, vp: Viewport) -> Self::Frame;

    /// Render an element on the current frame.
    fn render_element<R : resource::ResourceManager>(
        &self,
        resource_manager: &R,
        frame: &mut Self::Frame,
        data: &rendering::RenderData);

    // Flush the frame. Typically, swap buffers.
    fn flush_frame(&self, frame: Self::Frame);
}

#[derive(Copy, Clone)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
}
