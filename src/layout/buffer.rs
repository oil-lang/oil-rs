use std::slice;
use std::ops::Deref;
use super::LayoutBox;
use super::boxes::{LayoutBoxIterMut};

use style::{StyledNode};

// LayoutBuffer are of fixed sized.
pub struct LayoutBuffer(Box<[LayoutBox]>);

impl Deref for LayoutBuffer {
    type Target = [LayoutBox];

    fn deref<'a>(&'a self) -> &'a [LayoutBox] {
        self.0.deref()
    }
}

impl LayoutBuffer {

    pub fn new(style_tree: &StyledNode) -> LayoutBuffer {
        // First traversal: get size for one big allocation
        // TODO: Is it really faster to traverse first the tree once
        // to compute the total size to avoid reallocations ?
        let size = style_tree.tree_size();
        let mut buffer = Vec::with_capacity(size);

        // Add children with properties
        let res = LayoutBuffer::fill_buffer(&mut buffer, style_tree, true);
        assert_eq!(res as usize, size);

        LayoutBuffer(buffer.into_boxed_slice())
    }

    pub fn compute_layout(&mut self, max_width: f32, max_height: f32) {

        // First pass: compute default width
        for root in LayoutBoxIterMut::new(&mut self.0) {
            root.compute_layout_defaut_width(max_width);
        }

        // Second pass: compute auto margins and width auto
        for root in LayoutBoxIterMut::new(&mut self.0) {
            root.compute_layout_auto_width(max_width);
        }

        // Third pass: layout children and compute their height
        for root in LayoutBoxIterMut::new(&mut self.0) {
            root.compute_layout_height_and_position(max_height);
        }
    }

    pub fn iter(&self) -> slice::Iter<LayoutBox> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> slice::IterMut<LayoutBox> {
        self.0.iter_mut()
    }

    fn fill_buffer(
        vec: &mut Vec<LayoutBox>,
        style_tree: &StyledNode,
        last_child: bool) -> isize
    {
        let index = vec.len();
        let mut next_sibling: isize = 1;
        let mut kids = style_tree.kids.len();

        // Default next sibling
        unsafe {
            // Las child with children.
            if kids > 0 {
                vec.push(LayoutBox::new(style_tree, -1));
            // Last child with no more children.
            } else {
                vec.push(LayoutBox::new(style_tree, 0));
            }
        }


        for kid in &style_tree.kids {
            kids -= 1;
            next_sibling += LayoutBuffer::fill_buffer(vec, kid, kids == 0);
        }

        if !last_child {
            unsafe {
                vec.get_unchecked_mut(index).set_next_sibling(next_sibling);
            }
        }

        next_sibling
    }
}

// TODO: test with iterators / lifetime
