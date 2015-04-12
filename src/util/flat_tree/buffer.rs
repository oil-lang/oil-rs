use std::ops::Deref;
use super::TreeNode;
use super::FlatTreeIter;
use super::FlatTreeIterMut;
use super::HasChildren;

pub struct FlatTree<T>(Box<[TreeNode<T>]>);

impl<T> Deref for FlatTree<T> {
    type Target = [TreeNode<T>];

    fn deref<'a>(&'a self) -> &'a [TreeNode<T>] {
        self.0.deref()
    }
}

impl<T> FlatTree<T> {

    pub fn new<F, N>(root: &N, cap: usize, node_producer: F) -> FlatTree<T>
        where N: HasChildren,
              F: Fn(&N) -> Option<T>
    {
        let mut buffer = Vec::with_capacity(cap);

        FlatTree::fill_buffer(&mut buffer, root, &node_producer, true);

        FlatTree(buffer.into_boxed_slice())
    }

    pub fn tree_iter<'a>(&'a self) -> FlatTreeIter<'a, T> {
        FlatTreeIter::new(&self.0)
    }

    pub fn tree_iter_mut<'a>(&'a mut self) -> FlatTreeIterMut<'a, T> {
        FlatTreeIterMut::new(&mut self.0)
    }

    fn fill_buffer<F, N>(
        vec: &mut Vec<TreeNode<T>>,
        node: &N,
        node_producer: &F,
        last_child: bool) -> isize
        where N: HasChildren,
              F: Fn(&N) -> Option<T>
    {
        // Do we have a child here ?
        if let Some(new_child) = node_producer(node) {

            let index = vec.len();
            let mut next_sibling: isize = 1;
            let mut kids = node.children().len();

            // Default next sibling
            unsafe {
                // Las child with children.
                if kids > 0 {
                    vec.push(TreeNode::new(new_child, -1));
                // Last child with no more children.
                } else {
                    vec.push(TreeNode::new(new_child, 0));
                }
            }


            for kid in node.children() {
                kids -= 1;
                next_sibling += FlatTree::fill_buffer(
                    vec,
                    kid,
                    node_producer.clone(),
                    kids == 0
                );
            }

            if !last_child {
                unsafe {
                    vec.get_unchecked_mut(index).set_next_sibling(next_sibling);
                }
            }

            next_sibling

        } else {
            // Child is ignored and all its sub tree.
            0
        }
    }
}
