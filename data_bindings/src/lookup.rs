/// A property accessor represent
///
#[derive(Clone, Debug)]
pub struct PropertyAccessor<'a> {
    path: &'a str,
}


impl<'a> PropertyAccessor<'a> {

    /// A property accessor can only be created from a path
    /// of the form:
    /// ```txt
    ///     <name1>.<name2> [...] .<nameN>
    /// ```
    ///
    pub fn new(path: &'a str) -> PropertyAccessor<'a> {
        PropertyAccessor {
            path: path,
        }
    }

    /// Returns the name associated with that property accessor
    /// or `None` if the end of the path has been reached.
    pub fn name(&self) -> &'a str {
        self.path.find('.').map(|i| &self.path[i..]).unwrap_or(self.path)
    }

    /// Returns the next property accessor in the path.
    /// If the end is reached, then calling name on the property
    /// accessor created with that function will return `None`.
    pub fn next(&self) -> PropertyAccessor<'a> {
        PropertyAccessor {
            path: &self.path[self.path.find('.').unwrap_or(self.path.len())..self.path.len()],
        }
    }
}

/// This iterator generate from a PropertyAccessor
/// a sequence of
///
///     (String, PropertyAccessor)
///
/// where the first String is called the "prefix". It is a convenient
/// iterator that works well with a `Map`-like container where keys can
/// contain the separator `.` char.
///
/// # Example:
///
/// Given the PropertyAccessor `"settings.gui.window.scale"`,
/// when trying in a Hashmap to access the object behind the property,
/// we're going to try to reach in order:
///
///     LocalProperty("settings"),  PropertyAccessor("gui.window.scale")
///     LocalProperty("settings.gui"),  PropertyAccessor("window.scale")
///     LocalProperty("settings.gui.window"),  PropertyAccessor("scale")
///     LocalProperty("settings.gui.window.scale"), PropertyAccessor("")
///
/// This check only make sense at the Hashmap level because Rust type won't
/// contains in their name a `.`.
pub struct PrefixKeyIter<'a> {
    property_full_path: &'a str,
    position: i8,
}

impl <'a> PrefixKeyIter<'a> {
    pub fn new(property: PropertyAccessor<'a>) -> PrefixKeyIter<'a> {
        PrefixKeyIter {
            property_full_path: property.path,
            position: 0,
        }
    }
}

impl <'a> Iterator for PrefixKeyIter<'a> {
    type Item = (&'a str, PropertyAccessor<'a>);

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.position == 0 {
            self.position += 1;
            return Some(("", PropertyAccessor::new(self.property_full_path)));
        }
        let mut position = self.position;
        self.position += 1;
        let mut iterator = self.property_full_path.split(|c| {
            if c == '.' {
                position -= 1;
                if position == 0 {
                    return true;
                }
            }
            false
        });
        let prefix = match iterator.next() {
            None => return None,
            Some(a) => a,
        };
        let key = match iterator.next() {
            None => return None,
            Some(a) => a,
        };
        Some((prefix, PropertyAccessor::new(key)))
    }
}


#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn new_should_create_a_property_accessor_with_name_eq_to_some() {
        let a = PropertyAccessor::new("test");
        let b = PropertyAccessor::new("");
        assert_eq!(a.name(), "test");
        assert_eq!(b.name(), "");
    }

    #[test]
    fn next_should_create_a_property_accessor_returning_the_next_name_in_the_path() {
        let a = PropertyAccessor::new("foo.bar");
        let b = a.next();
        assert_eq!(a.name(), "foo");
        assert_eq!(b.name(), "bar");
        assert_eq!(a.name(), "foo");
    }
}
