use std::ptr;
use std::mem;
use super::LayoutBox;


/// Mutable iterator over LayoutBuffer
pub struct LayoutBoxIterMut {
    current: *mut LayoutBox,
}

impl LayoutBoxIterMut {
    pub fn new(boxes: &mut Box<[LayoutBox]>) -> LayoutBoxIterMut {
        let lbox = boxes.iter().next().unwrap();
        LayoutBoxIterMut {
            current: unsafe { mem::transmute(lbox) }
        }
    }

    pub unsafe fn new_with_childnode(lbox: & LayoutBox) -> LayoutBoxIterMut {
        let ptr: *mut LayoutBox = mem::transmute(lbox);
        LayoutBoxIterMut {
            current: ptr.offset(1)
        }
    }

    pub fn new_empty() -> LayoutBoxIterMut {
        LayoutBoxIterMut {
            current: ptr::null_mut()
        }
    }
}

impl<'a> Iterator for LayoutBoxIterMut {
    type Item = &'a mut LayoutBox;

    fn next(&mut self) -> Option<&mut LayoutBox> {
        if self.current.is_null() {
            None
        } else {
            let node = unsafe { &mut *self.current };
            if node.next_sibling > 0 {
                unsafe {
                    self.current = self.current.offset(node.next_sibling);
                }
            } else {
                self.current = ptr::null_mut();
            }
            Some(node)
        }
    }
}

/// Immutable iterator over LayoutBuffer
pub struct LayoutBoxIter {
    current: *const LayoutBox,
}

impl LayoutBoxIter {
    pub fn new(boxes: &Box<[LayoutBox]>) -> LayoutBoxIter {
        let lbox = boxes.iter().next().unwrap();
        LayoutBoxIter {
            current: unsafe { mem::transmute(lbox) }
        }
    }

    pub unsafe fn new_with_childnode(lbox: & LayoutBox) -> LayoutBoxIter {
        let ptr: *mut LayoutBox = mem::transmute(lbox);
        LayoutBoxIter {
            current: ptr.offset(1)
        }
    }

    pub fn new_empty() -> LayoutBoxIter {
        LayoutBoxIter {
            current: ptr::null()
        }
    }
}

impl<'a> Iterator for LayoutBoxIter {
    type Item = &'a LayoutBox;

    fn next(&mut self) -> Option<&LayoutBox> {
        if self.current.is_null() {
            None
        } else {
            let node = unsafe { &*self.current };
            if node.next_sibling > 0 {
                unsafe {
                    self.current = self.current.offset(node.next_sibling);
                }
            } else {
                self.current = ptr::null();
            }
            Some(node)
        }
    }
}
