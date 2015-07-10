use std::collections::HashMap;

use super::{StoreValue, ContextManager, DBCLookup};
use markup::{View, Template, NodeType};
use util::BufferFromTree;
use layout::LayoutBuffer;

pub struct DataBindingBuffer {
    bindings: BufferFromTree<DataBindingNode>,
    iterators: BufferFromTree<IteratorNode>,
    iterator_bindings: BufferFromTree<IteratorBindingNode>,
}

struct DataBindingNode {
    key: String,
    current: StoreValue,
}

struct IteratorNode {
    iterator: String,
    number: u32,
}

struct IteratorBindingNode {
    iterator: String,
    key: String,
    current: Vec<StoreValue>,
}

impl DataBindingNode {
    fn update(&mut self, context: &ContextManager, layout: &mut LayoutBuffer, lookup: usize) -> bool {
        match context.get_attribute(&self.key) {
            None => {
                println!("WARNING: Failed to update binding {}", self.key);
                false
            },
            Some(new_value) => {
                if new_value != self.current {
                    self.current = new_value;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn new(key: String) -> DataBindingNode {
        DataBindingNode {
            key: key,
            current: StoreValue::Integer(0),
        }
    }
}

impl IteratorNode {
    fn update(&mut self, context: &ContextManager, layout: &mut LayoutBuffer, lookup: usize) -> bool {
        match context.iterator_len(&self.iterator) {
            Err(e) => {
                println!("WARNING: Failed to update iterator {}: {}", self.iterator, e);
                false
            },
            Ok(new_value) => {
                if new_value != self.number {
                    self.number = new_value;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn new(it: String) -> IteratorNode {
        IteratorNode {
            iterator: it,
            number: 0,
        }
    }
}

impl IteratorBindingNode {
    fn update(&mut self, context: &ContextManager, layout: &mut LayoutBuffer, lookup: usize) -> bool {
        match context.compare_and_update(&self.iterator, &self.key, &mut self.current) {
            Ok(res) => res,
            Err(e) => {
                println!("ERROR: Failed to update repeat binding {} {}: {}", self.iterator, self.key, e);
                false
            }
        }
    }

    fn new(iterator: String, key: String) -> IteratorBindingNode {
        IteratorBindingNode {
            iterator: iterator,
            key: key,
            current: Vec::new(),
        }
    }
}

impl DataBindingBuffer {
    pub fn update(&mut self, context: &ContextManager, layout: &mut LayoutBuffer) -> bool {
        let mut has_changed = false;
        for (&lookup, node) in self.bindings.enumerate_lookup_indices_mut().unwrap() {
            if node.update(context, layout, lookup) {
                has_changed = true;
            }
        }
        for (&lookup, node) in self.iterators.enumerate_lookup_indices_mut().unwrap() {
            if node.update(context, layout, lookup) {
                has_changed = true;
            }
        }
        for (&lookup, node) in self.iterator_bindings.enumerate_lookup_indices_mut().unwrap() {
            if node.update(context, layout, lookup) {
                has_changed = true;
            }
        }
        has_changed
    }

    pub fn new(view: &View, templates: &HashMap<String, Template>) -> DataBindingBuffer {
        let bindings = BufferFromTree::new_with_lookup_table(view, 0, |node| {
            match node.node_type {
                NodeType::Binding(ref binding) => {
                    Some(DataBindingNode::new(binding.clone()))
                }
                _ => None
            }
        });
        let iterators = BufferFromTree::new_with_lookup_table(view, 0, |node| {
            match node.node_type {
                // NodeType::Repeat(RepeatData { ref template_name, ref iter }) => {
                //     Some(IteratorNode::new(iterator.clone()))
                // }
                _ => None
            }
        });
        let iterator_bindings = BufferFromTree::new_with_lookup_table(view, 0, |node| {
            match node.node_type {
                // NodeType::RepeatBinding(RepeatBindingData{ref iter, ref key}) => {
                //     Some(IteratorBindingNode::new(iter.clone(), key.clone()))
                // }
                _ => None
            }
        });
        DataBindingBuffer {
            bindings: bindings,
            iterators: iterators,
            iterator_bindings: iterator_bindings,
        }
    }
}
