use std::fmt;

#[derive(Debug)]
pub struct Error {
    row: usize,
    col: usize,
    msg: String
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}: {}", self.row + 1, self.col + 1, self.msg)
    }
}


impl Error {
    /// Creates a new error using position information from the provided
    /// `HasPosition` object and a message.
    #[inline]
    pub fn new(row: usize, col: usize, msg: String) -> Error {
        Error {
            row: row,
            col: col,
            msg: msg
        }
    }
}
