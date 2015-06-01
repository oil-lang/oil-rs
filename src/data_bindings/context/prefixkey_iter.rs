

/// An iterator over the different combinations of prefix-key
pub struct PrefixKeyIter<'a> {
    data: &'a str,
    position: i8,
}

impl <'a> PrefixKeyIter<'a> {
    pub fn new(data: &'a str) -> PrefixKeyIter<'a> {
        PrefixKeyIter {
            data: data,
            position: 0,
        }
    }
}

impl <'a> Iterator for PrefixKeyIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.position == 0 {
            self.position += 1;
            return Some(("", self.data));
        }
        let mut position = self.position;
        self.position += 1;
        let mut iterator = self.data.split(|c| {
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
        Some((prefix,key))
    }
}
