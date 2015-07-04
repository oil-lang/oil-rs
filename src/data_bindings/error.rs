use std::error::Error;

pub type BindingResult<T> = Result<T,DataBindingError>;

#[derive(Clone,Eq,PartialEq,Debug)]
pub enum DataBindingError {
    DanglingReference(String),
    IteratorNotFound(String),
    KeyNotFound(String),
    ViewNotFound,
}

impl Error for DataBindingError {
    fn description(&self) -> &str {
        match *self {
            DataBindingError::DanglingReference(_) => "Dangling data binding reference",
            DataBindingError::IteratorNotFound(_) => "Repeat iterator not found",
            DataBindingError::KeyNotFound(_) => "Key not found",
            DataBindingError::ViewNotFound => "View not found",
        }
    }
}

impl ::std::fmt::Display for DataBindingError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        try!(self.description().fmt(f));
        let details = match *self {
            DataBindingError::DanglingReference(ref s) => Some(s),
            DataBindingError::IteratorNotFound(ref s) => Some(s),
            DataBindingError::KeyNotFound(ref s) => Some(s),
            DataBindingError::ViewNotFound => None,
        };

        if let Some(d) = details {
            d.fmt(f)
        } else {
            Ok(())
        }
    }
}
