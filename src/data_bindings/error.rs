use std::error::Error;

pub type BindingResult<T> = Result<T,DataBindingError>;

#[derive(Clone,Eq,PartialEq,Debug)]
pub enum DataBindingError {
    DanglingReference(String),
    IteratorNotFound(String),
    KeyNotFound(String),
    ViewNotFound(String),
}

impl Error for DataBindingError {
    fn description(&self) -> &str {
        match *self {
            DataBindingError::DanglingReference(_) => "Dangling data binding reference",
            DataBindingError::IteratorNotFound(_) => "Repeat iterator not found",
            DataBindingError::KeyNotFound(_) => "Key not found",
            DataBindingError::ViewNotFound(_) => "View not found",
        }
    }
}

impl ::std::fmt::Display for DataBindingError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        try!(self.description().fmt(f));
        let details = match *self {
            DataBindingError::DanglingReference(ref s) => s,
            DataBindingError::IteratorNotFound(ref s) => s,
            DataBindingError::KeyNotFound(ref s) => s,
            DataBindingError::ViewNotFound(ref s) => s,
        };
        details.fmt(f)
    }
}
