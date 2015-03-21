
#![feature(core, old_io)]
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

#[cfg(feature = "use_glium")]
pub mod glium;

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

pub trait RenderContext {
    fn renderElement<B, R>(&mut self, boxi: &B, data: &R)
        where B: layout::Box, R: rendering::Material;
}
