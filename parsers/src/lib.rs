#![feature(core)]
#![feature(io)]
#![feature(plugin)]
#![plugin(phf_macros)]

extern crate xml;
extern crate phf;
extern crate num;

extern crate uil_shared;

pub mod style;
pub mod markup;
pub mod deps;

pub use self::report::ErrorReporter;
pub use self::report::StdOutErrorReporter;
pub use self::report::EmptyErrorReporter;

mod parsing;
mod report;
