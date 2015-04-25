use std::ops::Deref;
use std::ops::DerefMut;
use super::TreeNode;
use super::FlatTreeIter;
use super::FlatTreeIterMut;
use super::HasChildren;
use std::mem;

pub struct FlatTree<T>(Box<[TreeNode<T>]>);

impl<T> Deref for FlatTree<T> {
    type Target = [TreeNode<T>];

    fn deref<'a>(&'a self) -> &'a [TreeNode<T>] {
        self.0.deref()
    }
}

impl<T> DerefMut for FlatTree<T> {

    fn deref_mut<'a>(&'a mut self) -> &'a mut [TreeNode<T>] {
        self.0.deref_mut()
    }
}

impl<T> FlatTree<T> {

    pub fn new<F, N>(root: &N, cap: usize, node_producer: F) -> FlatTree<T>
        where N: HasChildren,
              F: Fn(&N) -> Option<T>
    {
        let mut buffer = Vec::with_capacity(cap);

        FlatTree::fill_buffer(
            &mut buffer,
            &mut None,
            &mut 0,
            root,
            &node_producer,
            true
        );

        FlatTree(buffer.into_boxed_slice())
    }

    pub fn new_with_lookup_table<F, N>(
        root: &N,
        cap: usize,
        node_producer: F) -> (FlatTree<T>, Box<[usize]>)
        where N: HasChildren,
              F: Fn(&N) -> Option<T>
    {
        let mut buffer = Vec::with_capacity(cap);
        let mut lookup_table = Some(Vec::with_capacity(cap));

        FlatTree::fill_buffer(
            &mut buffer,
            &mut lookup_table,
            &mut 0,
            root,
            &node_producer,
            true
        );

        (FlatTree(buffer.into_boxed_slice()), lookup_table.unwrap().into_boxed_slice())
    }

    /// Returns the index of the given node in this tree.
    ///
    /// # Panics
    ///
    /// This method panics if the node given does not belong to this tree.
    pub fn node_as_index(&self, node: &TreeNode<T>) -> isize {
        let index = (&self.0.deref()[0] as *const TreeNode<T> as usize
            - node as *const TreeNode<T> as usize) as isize /
            mem::size_of::<TreeNode<T>>() as isize;
        // If the diff is not in the range [0, len) then this is a bug.
        assert!((index as usize) < self.0.len());
        assert!(index > 0);
        // Return diff
        index
    }

    pub fn tree_iter<'a>(&'a self) -> FlatTreeIter<'a, T> {
        FlatTreeIter::new(&self.0)
    }

    pub fn tree_iter_mut<'a>(&'a mut self) -> FlatTreeIterMut<'a, T> {
        FlatTreeIterMut::new(&mut self.0)
    }

    fn fill_buffer<F, N>(
        vec: &mut Vec<TreeNode<T>>,
        lookup_table: &mut Option<Vec<usize>>,
        current_index: &mut usize,
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

            // Set values for lookup_table
            match lookup_table.as_mut() {
                Some(ref mut lookup_indices) => {
                    lookup_indices.push(*current_index);
                },
                _ => (),
            }
            *current_index += 1;

            for kid in node.children() {
                kids -= 1;
                next_sibling += FlatTree::fill_buffer(
                    vec,
                    lookup_table,
                    current_index,
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
            // Increment the index.
            increment_index(current_index, node);

            // Returns the next_sibling increment value
            0
        }
    }

}

// ======================================== //
//                  HELPERS                 //
// ======================================== //

fn increment_index<N>(current_index: &mut usize, node: &N)
    where N: HasChildren
{
    *current_index += 1;

    for kid in node.children() {
        increment_index(current_index, kid);
    }
}
