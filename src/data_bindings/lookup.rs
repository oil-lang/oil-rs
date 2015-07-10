use std::borrow::Cow;
use std::borrow::IntoCow;

/// A property accessor represent
///
#[derive(Clone, Debug)]
pub struct PropertyAccessor {
    path: Cow<'static, str>,
    name_index: usize,
}


impl <T> From<T> for PropertyAccessor
    where T: Into<String>
{
    fn from(s: T) -> PropertyAccessor {
        PropertyAccessor::new(s)
    }
}

impl PropertyAccessor {

    /// A property accessor can only be created from a path
    /// of the form:
    /// ```txt
    ///     <name1>.<name2> [...] .<nameN>
    /// ```
    ///
    pub fn new<T: Into<String>>(path: T) -> PropertyAccessor {
        PropertyAccessor {
            path: path.into().into_cow(),
            name_index: 0,
        }
    }

    /// Returns the name associated with that property accessor
    /// or `None` if the end of the path has been reached.
    pub fn name<'a>(&'a self) -> Option<&'a str> {
        self.path.split('.').nth(self.name_index)
    }

    /// Returns the next property accessor in the path.
    /// If the end is reached, then calling name on the property
    /// accessor created with that function will return `None`.
    pub fn next(&self) -> PropertyAccessor {
        PropertyAccessor {
            path: self.path.clone(),
            name_index: self.name_index + 1,
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn new_should_create_a_property_accessor_with_name_eq_to_Some() {
        let a = PropertyAccessor::new("test");
        let b = PropertyAccessor::new("");
        assert_eq!(a.name(), Some("test"));
        assert_eq!(b.name(), Some(""));
    }

    #[test]
    fn next_should_create_a_property_accessor_returning_the_next_name_in_the_path() {
        let a = PropertyAccessor::new("foo.bar");
        assert_eq!(a.next().name(), Some("bar"));
        assert_eq!(a.name(), Some("foo"));
    }
}
