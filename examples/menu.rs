extern crate glutin;
extern crate glium;
extern crate image;
extern crate clock_ticks;
extern crate uil;

mod util;

fn main() {

    util::run_example(
        "uil - menu example",
        "./examples/menu.markup",
        "./examples/menu.deps",
        "./examples/menu.style"
    );
}
