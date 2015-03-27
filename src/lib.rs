#![feature(convert)]
#![feature(core, io)]
#![feature(collections)]
#![feature(plugin)]
#![plugin(phf_macros)]

#[macro_use]
extern crate bitflags;
extern crate xml;
extern crate phf;

#[cfg(feature = "use_glium")]
#[macro_use]
extern crate glium;
#[cfg(feature = "use_glium")]
extern crate image;

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

mod parsing;
mod report;
mod asset;
mod router;
mod view;

pub trait RenderBackbend {
    fn render_element<B, R>(&mut self, boxi: &B, data: &R)
        where B: layout::Box, R: rendering::Material;
}

#[derive(Copy)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
}
