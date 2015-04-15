
pub trait ErrorReporter: Clone {

    fn log(&self, msg: String);
}

#[derive(Clone)]
pub struct StdOutErrorReporter;
#[derive(Clone)]
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
