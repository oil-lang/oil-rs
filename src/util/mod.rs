use std::cmp::Ord;
use std::cmp::Ordering;

pub mod flat_tree;

#[derive(PartialOrd, PartialEq)]
pub struct F32Ord(pub f32);

impl Ord for F32Ord {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 < other.0 {
            Ordering::Less
        } else if self.0 == other.0 {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }
}

impl Eq for F32Ord {}

#[inline]
pub fn ref_eq<T>(a: &T, b: &T) -> bool {
    a as *const T == b as *const T
}
