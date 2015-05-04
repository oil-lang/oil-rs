use std::ops::Deref;
use std::ops::DerefMut;
use std::ptr;
use std::mem;
use std::marker;

pub use self::buffer::FlatTree;

mod buffer;

pub struct TreeNode<T> {
    data: T,
    next_sibling: isize,
}

impl<T> Deref for TreeNode<T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        &self.data
    }
}

impl<T> DerefMut for TreeNode<T> {

    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.data
    }
}

impl<T> TreeNode<T> {

    unsafe fn new(data: T, next_sibling: isize) -> TreeNode<T> {
        TreeNode {
            data: data,
            next_sibling: next_sibling,
        }
    }

    #[inline]
    unsafe fn set_next_sibling(&mut self, next_sibling: isize) {
        self.next_sibling = next_sibling;
    }

    pub fn children<'a>(&'a self) -> FlatTreeIter<'a, T> {
        if self.next_sibling > 1 || self.next_sibling == -1 {
            unsafe { FlatTreeIter::new_with_firstchild(self) }
        } else {
            FlatTreeIter::new_empty()
        }
    }

    pub fn children_mut<'a>(&'a self) -> FlatTreeIterMut<'a, T> {
        if self.next_sibling > 1 || self.next_sibling == -1 {
            unsafe { FlatTreeIterMut::new_with_firstchild(self) }
        } else {
            FlatTreeIterMut::new_empty()
        }
    }
}


/// Mutable iterator over LayoutBuffer
pub struct FlatTreeIterMut<'a, T: 'a> {
    current: *mut TreeNode<T>,
    _marker: marker::PhantomData<&'a mut TreeNode<T>>,
}

impl<'a, T> FlatTreeIterMut<'a, T> {
    fn new(flat: &'a mut Box<[TreeNode<T>]>) -> FlatTreeIterMut<'a, T> {
        let tn: &mut TreeNode<T> = flat.first_mut().unwrap();
        FlatTreeIterMut {
            current: tn,
            _marker: marker::PhantomData
        }
    }

    unsafe fn new_with_firstchild(tn: &'a TreeNode<T>) -> FlatTreeIterMut<'a, T> {
        let ptr: *mut TreeNode<T> = mem::transmute(tn);
        FlatTreeIterMut {
            current: ptr.offset(1),
            _marker: marker::PhantomData
        }
    }

    fn new_empty() -> FlatTreeIterMut<'a, T> {
        FlatTreeIterMut {
            current: ptr::null_mut(),
            _marker: marker::PhantomData
        }
    }
}

impl<'a, T> Iterator for FlatTreeIterMut<'a, T> {
    type Item = &'a mut TreeNode<T>;

    fn next(&mut self) -> Option<&'a mut TreeNode<T>> {
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

/// Immutable iterator over FlatTree
pub struct FlatTreeIter<'a, T: 'a> {
    current: *const TreeNode<T>,
    _marker: marker::PhantomData<&'a TreeNode<T>>,
}

impl<'a, T> FlatTreeIter<'a, T> {

    fn new(flat: &'a Box<[TreeNode<T>]>) -> FlatTreeIter<'a, T> {
        let tn = flat.iter().next().unwrap();
        FlatTreeIter {
            current: unsafe { mem::transmute(tn) },
            _marker: marker::PhantomData
        }
    }

    unsafe fn new_with_firstchild(tn: &'a TreeNode<T>) -> FlatTreeIter<'a, T> {
        let ptr: *mut TreeNode<T> = mem::transmute(tn);
        FlatTreeIter {
            current: ptr.offset(1),
            _marker: marker::PhantomData
        }
    }

    fn new_empty() -> FlatTreeIter<'a, T> {
        FlatTreeIter {
            current: ptr::null(),
            _marker: marker::PhantomData
        }
    }
}

impl<'a, T> Iterator for FlatTreeIter<'a, T> {
    type Item = &'a TreeNode<T>;

    fn next(&mut self) -> Option<&'a TreeNode<T>> {
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
