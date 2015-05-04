extern crate glutin;
extern crate glium;
extern crate image;
extern crate clock_ticks;
extern crate uil;

mod util;

fn main() {

    util::run_example(
        "uil - shop inventory example",
        "./examples/shop_inventory.markup",
        "./examples/shop_inventory.deps",
        "./examples/shop_inventory.style"
    );
}
