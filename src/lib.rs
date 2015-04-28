#![feature(collections)]
#![feature(core)]
#![feature(std_misc)]
#![feature(plugin)]
#![feature(alloc)]
#![plugin(phf_macros)]

#[macro_use]
extern crate bitflags;
extern crate phf;
extern crate num;

#[macro_use]
extern crate glium;
extern crate image;
extern crate cgmath;
extern crate uil_parsers;
extern crate uil_shared;

#[cfg(test)]
extern crate glutin;

pub mod markup;
pub mod style;
pub mod deps;
pub mod rendering;
pub mod resource;

// Reexport
pub use uil_parsers::ErrorReporter;
pub use uil_parsers::StdOutErrorReporter;
pub use uil_parsers::EmptyErrorReporter;
pub use self::router::Router;
pub use self::rendering::View;
pub use self::data_bindings::DataBinderContext;
pub use self::data_bindings::DBStore;

mod layout;
mod router;
mod util;
mod focus;
mod data_bindings;

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
