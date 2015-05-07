extern crate glutin;
extern crate glium;
extern crate image;
extern crate clock_ticks;
extern crate oil;

mod util;

fn main() {

    util::run_example(
        "oil - list example",
        "./examples/list.markup",
        "./examples/list.deps",
        "./examples/list.style"
    );
}
