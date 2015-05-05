extern crate glutin;
extern crate glium;
extern crate image;
extern crate clock_ticks;
extern crate oil;

mod util;

fn main() {

    util::run_example(
        "oil - shop inventory example",
        "./examples/shop_inventory.markup",
        "./examples/shop_inventory.deps",
        "./examples/shop_inventory.style"
    );
}
