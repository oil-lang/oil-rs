
pub mod cache;

pub use self::unit::Au;
mod unit;

pub fn float_to_fixed(before: usize, f: f64) -> i32 {
    ((1i32 << before) as f64 * f) as i32
}

pub fn fixed_to_float(before: usize, f: i32) -> f64 {
    f as f64 * 1.0f64 / ((1i32 << before) as f64)
}

use libc::{c_void, size_t};

extern {
    // Get the size of a heap block.
    //
    // Ideally Rust would expose a function like this in std::rt::heap, which would avoid the
    // jemalloc dependence.
    //
    // The C prototype is `je_malloc_usable_size(JEMALLOC_USABLE_SIZE_CONST void *ptr)`. On some
    // platforms `JEMALLOC_USABLE_SIZE_CONST` is `const` and on some it is empty. But in practice
    // this function doesn't modify the contents of the block that `ptr` points to, so we use
    // `*const c_void` here.
    fn je_malloc_usable_size(ptr: *const c_void) -> size_t;
}

// A wrapper for je_malloc_usable_size that handles `EMPTY` and returns `usize`.
pub fn heap_size_of(ptr: *const c_void) -> usize {
    if ptr == ::std::rt::heap::EMPTY as *const c_void {
        0
    } else {
        unsafe { je_malloc_usable_size(ptr) as usize }
    }
}
