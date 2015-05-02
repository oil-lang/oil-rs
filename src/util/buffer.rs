use std::iter::Zip;
use std::slice::{Iter, IterMut};
use std::ops::RangeFrom;
use std::ops::Deref;

use super::HasChildren;

pub struct BufferFromTree<T>{
    buffer: Box<[T]>,
    lookup_indices: Option<Box<[usize]>>,
}

impl<T> Deref for BufferFromTree<T> {
    type Target = [T];

    fn deref<'a>(&'a self) -> &'a [T] {
        self.buffer.deref()
    }
}

impl<T> BufferFromTree<T> {

    pub fn new<F, N>(root: &N, cap: usize, node_producer: F)
        -> BufferFromTree<T>
        where N: HasChildren,
              F: Fn(&N) -> Option<T>
    {
        let mut buffer = Vec::with_capacity(cap);

        BufferFromTree::fill_buffer(
            &mut buffer,
            &mut None,
            &mut 0,
            root,
            &node_producer
        );

        BufferFromTree {
            buffer: buffer.into_boxed_slice(),
            lookup_indices: None,
        }
    }

    pub fn new_with_lookup_table<F, N>(root: &N, cap: usize, node_producer: F)
        -> BufferFromTree<T>
        where N: HasChildren,
              F: Fn(&N) -> Option<T>
    {
        let mut buffer = Vec::with_capacity(cap);
        let mut lookup_table = Some(Vec::with_capacity(cap));

        BufferFromTree::fill_buffer(
            &mut buffer,
            &mut lookup_table,
            &mut 0,
            root,
            &node_producer
        );

        BufferFromTree {
            buffer: buffer.into_boxed_slice(),
            lookup_indices: Some(lookup_table.unwrap().into_boxed_slice()),
        }
    }

    pub fn from_buffer<U, F>(from: &BufferFromTree<U>, mut converter: F) -> BufferFromTree<T>
        where F: FnMut(&U) -> Option<T>
    {
        let mut buffer = Vec::with_capacity(from.buffer.len());
        let mut lookup_indices = Vec::with_capacity(from.buffer.len());

        if from.lookup_indices.is_some() {
            for (&i, f) in from.enumerate_lookup_indices().unwrap() {
                if let Some(c) = converter(f) {
                    buffer.push(c);
                    lookup_indices.push(i);
                }
            }
        } else {
            for (i, f) in from.enumerate() {
                if let Some(c) = converter(f) {
                    buffer.push(c);
                    lookup_indices.push(i);
                }
            }
        }

        BufferFromTree {
            buffer: buffer.into_boxed_slice(),
            lookup_indices: Some(lookup_indices.into_boxed_slice())
        }
    }

    pub fn enumerate_lookup_indices<'a>(&'a self)
        -> Option<Zip<Iter<usize>, Iter<'a, T>>>
    {
        if let Some(ref tb) = self.lookup_indices {
            Some(tb.iter().zip(self.buffer.iter()))
        } else {
            None
        }
    }

    pub fn enumerate_lookup_indices_mut<'a>(&'a mut self)
        -> Option<Zip<Iter<usize>, IterMut<'a, T>>>
    {
        if let Some(ref tb) = self.lookup_indices {
            Some(tb.iter().zip(self.buffer.iter_mut()))
        } else {
            None
        }
    }

    pub fn enumerate<'a>(&'a self)
        -> Zip<RangeFrom<usize>, Iter<'a, T>>
    {
        (0..).zip(self.buffer.iter())
    }

    pub fn enumerate_mut<'a>(& 'a mut self)
        -> Zip<RangeFrom<usize>, IterMut<'a,T>>
    {
        (0..).zip(self.buffer.iter_mut())
    }

    fn fill_buffer<F, N>(
        vec: &mut Vec<T>,
        lookup_table: &mut Option<Vec<usize>>,
        current_index: &mut usize,
        node: &N,
        node_producer: &F)
        where N: HasChildren,
              F: Fn(&N) -> Option<T>
    {
        // Do we have a child here ?
        if let Some(new_child) = node_producer(node) {

            vec.push(new_child);

            match lookup_table.as_mut() {
                Some(ref mut lookup_indices) => {
                    lookup_indices.push(
                        *current_index
                    );
                }
                _ => (),
            }
        }

        *current_index += 1;

        for kid in node.children() {
            BufferFromTree::fill_buffer(
                vec,
                lookup_table,
                current_index,
                kid,
                node_producer
            );
        }
    }

}

// ======================================== //
//                   TESTS                  //
// ======================================== //

#[cfg(test)]
mod test {

    use std::cell::Cell;
    use super::*;
    use util::HasChildren;

    #[derive(Default)]
    struct SomeTree {
        kids: Vec<SomeTree>,
    }

    struct NodeData;

    impl HasChildren for SomeTree {
        fn children<'a>(&'a self) -> &'a [SomeTree] {
            &self.kids
        }
    }

    fn create_buffer_from_some_tree() -> BufferFromTree<NodeData>
    {
        let mut root = SomeTree::default();
        let mut child = SomeTree::default();
        root.kids.push(SomeTree::default());
        child.kids.push(SomeTree::default());
        child.kids.push(SomeTree::default());
        root.kids.push(child);
        root.kids.push(SomeTree::default());
        let index = Cell::new(0);
        let producer = |_: &SomeTree| {
            if index.get() % 2 == 0 {
                index.set(index.get() + 1);
                Some(NodeData)
            } else {
                index.set(index.get() + 1);
                None
            }
        };

        BufferFromTree::new_with_lookup_table(&root, 8, &producer)
    }

    #[test]
    fn lookup_table_length_should_eq_buffer_length() {
        let flatmapping = create_buffer_from_some_tree();
        let test = flatmapping.lookup_indices.unwrap();
        assert_eq!(test.len(), flatmapping.buffer.len());
    }

    #[test]
    fn lookup_table_contains_correct_indices() {
        let flatmapping = create_buffer_from_some_tree();
        let test = flatmapping.lookup_indices.unwrap();
        assert_eq!(test[0], 0);
        assert_eq!(test[1], 2);
        assert_eq!(test[2], 4);
        assert_eq!(test.len(), 3);
    }

    #[test]
    fn enumerate_mut_should_give_buffer_index() {
        let mut flatmapping = create_buffer_from_some_tree();
        let mut index = 0;
        for (i, _) in flatmapping.enumerate_mut() {
            assert_eq!(i, index);
            index += 1;
        }
    }

    #[test]
    fn enumerate_lookup_indices_mut_should_give_original_index() {
        let mut flatmapping = create_buffer_from_some_tree();
        let mut index = 0;
        let mut indices = vec![0; flatmapping.buffer.len()].into_boxed_slice();
        indices.clone_from_slice(flatmapping.lookup_indices.as_ref().unwrap());
        for (&i, _) in flatmapping.enumerate_lookup_indices_mut().unwrap() {
            assert_eq!(i, indices[index]);
            index += 1;
        }
    }

}
