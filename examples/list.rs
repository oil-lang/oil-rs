extern crate glutin;
extern crate glium;
extern crate image;
extern crate clock_ticks;
extern crate uil;

mod util;

fn main() {

    util::run_example(
        "uil - list example",
        "./examples/list.markup",
        "./examples/list.deps",
        "./examples/list.style"
    );
}
