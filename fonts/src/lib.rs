#![feature(box_syntax)]
#![feature(heap_api)]
#![feature(box_raw)]
#![feature(hashmap_hasher)]

#[macro_use] extern crate bitflags;
#[macro_use] extern crate log;
extern crate smallvec;
extern crate harfbuzz;
extern crate freetype_sys;
extern crate string_cache;
extern crate euclid;
extern crate libc;
extern crate rand;

mod shaper;
mod freetype;
mod font_context;
mod util;
mod glyph;
mod font;
