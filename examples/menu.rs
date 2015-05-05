extern crate glutin;
extern crate glium;
extern crate image;
extern crate clock_ticks;
extern crate oil;

mod util;

fn main() {

    util::run_example(
        "oil - menu example",
        "./examples/menu.markup",
        "./examples/menu.deps",
        "./examples/menu.style"
    );
}
