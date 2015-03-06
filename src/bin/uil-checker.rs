#![feature(old_path, old_io)]

extern crate uil;

use std::old_io::{File, BufferedReader};

fn main() {
    {
        let file = File::open(&Path::new("assets/markup/test.xml")).unwrap();
        let reader = BufferedReader::new(file);

        uil::markup::parse(uil::StdOutErrorReporter, reader);
    }
    let styledefs = {
        let file = File::open(&Path::new("assets/deps/test.deps")).unwrap();
        let reader = BufferedReader::new(file);

        uil::deps::parse(uil::StdOutErrorReporter, reader)
    };
    {
        let file = File::open(&Path::new("assets/style/test.style")).unwrap();
        let reader = BufferedReader::new(file);

        uil::style::parse(uil::StdOutErrorReporter, reader, &styledefs);
    }
}
