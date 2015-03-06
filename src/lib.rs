
#![feature(core, old_io)]
#![feature(plugin)]
#![plugin(phf_macros)]

#[macro_use]
extern crate bitflags;
extern crate xml;
extern crate phf;

// TODO: Export only minimum
pub mod markup;
pub mod style;
pub mod deps;
pub mod layout;

pub use self::report::ErrorReporter;
pub use self::report::StdOutErrorReporter;
pub use self::report::EmptyErrorReporter;

mod parsing;
mod report;
mod asset;
