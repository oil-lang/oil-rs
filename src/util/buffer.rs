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

    pub fn enumerate_lookup_indices_mut<'a>(&'a mut self)
        -> Option<Zip<Iter<usize>, IterMut<'a, T>>>
    {
        if let Some(ref tb) = self.lookup_indices {
            Some(tb.iter().zip(self.buffer.iter_mut()))
        } else {
            None
        }
    }

    pub fn enumerate_mut<'a>(& 'a mut self) -> Zip<RangeFrom<usize>, IterMut<'a,T>> {
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
