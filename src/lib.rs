#![feature(core, io)]
#![feature(plugin)]
#![plugin(phf_macros)]

#[macro_use]
extern crate bitflags;
extern crate xml;
extern crate phf;

#[macro_use]
extern crate glium;
extern crate image;
extern crate cgmath;

#[cfg(test)]
extern crate glutin;

// TODO: Export only minimum
pub mod markup;
pub mod style;
pub mod deps;
pub mod rendering;
pub mod resource;

pub use self::report::ErrorReporter;
pub use self::report::StdOutErrorReporter;
pub use self::report::EmptyErrorReporter;
pub use self::router::Router;
pub use self::rendering::View;

mod layout;
mod parsing;
mod report;
mod asset;
mod router;
mod util;

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
