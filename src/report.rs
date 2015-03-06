
pub trait ErrorReporter: Copy {

    fn log(&self, msg: String);
}

#[derive(Copy)]
pub struct StdOutErrorReporter;
#[derive(Copy)]
pub struct EmptyErrorReporter;


impl ErrorReporter for StdOutErrorReporter {

    #[inline]
    fn log(&self, msg: String) {
        println!("{}", msg);
    }
}

impl ErrorReporter for EmptyErrorReporter {

    #[inline]
    #[allow(unused_variables)]
    fn log(&self, msg: String) {
        // Does nothing
    }
}
