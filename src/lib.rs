#![feature(convert)]
#![feature(core, io)]
#![feature(collections)]
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

// TODO: Export only minimum
pub mod markup;
pub mod style;
pub mod deps;
pub mod layout;
pub mod rendering;
pub mod backend;

pub use self::report::ErrorReporter;
pub use self::report::StdOutErrorReporter;
pub use self::report::EmptyErrorReporter;
pub use self::router::Router;
pub use self::view::View;
pub use self::resource::ResourceManager;

mod parsing;
mod report;
mod asset;
mod router;
mod view;
mod resource;

pub trait RenderBackbend {
    type Frame;

    /// Prepare the frame for the current rendering step.
    fn prepare_frame(&mut self, vp: Viewport) -> Self::Frame;

    /// Render an element on the current frame.
    fn render_element(
        &self,
        resource_manager: &ResourceManager,
        frame: &mut Self::Frame,
        data: &rendering::RenderData);

    // Flush the frame. Typically, swap buffers.
    fn flush_frame(&self, frame: Self::Frame);
}

#[derive(Copy)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
}
