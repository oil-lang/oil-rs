use std::ptr;
use std::mem;
use std::marker;
use super::LayoutBox;
use layout::LayoutBuffer;

/// Mutable iterator over LayoutBuffer
pub struct LayoutBoxIterMut<'a> {
    current: *mut LayoutBox,
    _marker: marker::PhantomData<&'a mut LayoutBuffer>,
}

impl<'s> LayoutBoxIterMut<'s> {
    pub fn new<'a>(boxes: &'a mut Box<[LayoutBox]>) -> LayoutBoxIterMut<'a> {
        let lbox = boxes.iter().next().unwrap();
        LayoutBoxIterMut {
            current: unsafe { mem::transmute(lbox) },
            _marker: marker::PhantomData
        }
    }

    pub unsafe fn new_with_childnode<'a>(lbox: &'a LayoutBox) -> LayoutBoxIterMut<'a> {
        let ptr: *mut LayoutBox = mem::transmute(lbox);
        LayoutBoxIterMut {
            current: ptr.offset(1),
            _marker: marker::PhantomData
        }
    }

    pub fn new_empty<'a>() -> LayoutBoxIterMut<'a> {
        LayoutBoxIterMut {
            current: ptr::null_mut(),
            _marker: marker::PhantomData
        }
    }
}

impl<'b, 'a> Iterator for LayoutBoxIterMut<'b> {
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
pub struct LayoutBoxIter<'a> {
    current: *const LayoutBox,
    _marker: marker::PhantomData<&'a LayoutBuffer>,
}

impl<'s> LayoutBoxIter<'s> {
    pub fn new<'a>(boxes: &'a Box<[LayoutBox]>) -> LayoutBoxIter<'a> {
        let lbox = boxes.iter().next().unwrap();
        LayoutBoxIter {
            current: unsafe { mem::transmute(lbox) },
            _marker: marker::PhantomData
        }
    }

    pub unsafe fn new_with_childnode<'a>(lbox: &'a LayoutBox) -> LayoutBoxIter<'a> {
        let ptr: *mut LayoutBox = mem::transmute(lbox);
        LayoutBoxIter {
            current: ptr.offset(1),
            _marker: marker::PhantomData
        }
    }

    pub fn new_empty<'a>() -> LayoutBoxIter<'a> {
        LayoutBoxIter {
            current: ptr::null(),
            _marker: marker::PhantomData
        }
    }
}

impl<'a, 'b> Iterator for LayoutBoxIter<'b> {
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
