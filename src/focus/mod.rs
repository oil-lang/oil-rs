use std::f32;
use std::ptr;
use std::mem;
use std::ops::{Index, Deref};

use util::flat_tree::{FlatTree, TreeNode};
use layout::{LayoutBuffer, Rect};
use markup::Node;
use self::tagged_tree::TaggedNode;
use std::default::Default;
use self::direction::{Axis, Cursor};

mod tagged_tree;
mod direction;

pub struct FocusedElement {
    focus_node: isize,
    cursor: Cursor,
}

pub struct FocusAcceptor {
    // The parent of this node.
    parent: *const FocusNode,
    is_acceptor: bool,
    line_number: usize,
    bounds: Rect,
}

impl FocusAcceptor {

    fn new(node: &TaggedNode) -> FocusAcceptor {
        FocusAcceptor {
            parent: ptr::null_mut(),
            line_number: 0,
            is_acceptor: node.is_acceptor,
            bounds: Default::default(),
        }
    }

    pub fn parent(&self) -> Option<&FocusNode> {
        if self.parent.is_null() {
            None
        } else {
            Some(unsafe { mem::transmute(self.parent) })
        }
    }
}

pub type FocusNode = TreeNode<FocusAcceptor>;

pub struct FocusBuffer {
    buffer: FlatTree<FocusAcceptor>,
}

impl Deref for FocusBuffer {
    type Target = FlatTree<FocusAcceptor>;

    fn deref<'a>(&'a self) -> &'a FlatTree<FocusAcceptor> {
        &self.buffer
    }
}

impl Index<isize> for FocusBuffer {

    type Output = FocusNode;

    fn index<'a>(&'a self, _index: isize) -> &'a FocusNode {
        assert!(_index > 0);
        &self.buffer[_index as usize]
    }
}

impl FocusBuffer {

    pub fn new(root: &Node) -> FocusBuffer {

        let tagged_tree = TaggedNode::new(root);

        let mut tree = FlatTree::new_with_lookup_table(&tagged_tree, 10, converter);

        // Resolve parents
        for node in tree.tree_iter_mut() {
            unsafe {
                resolve_parent(node, ptr::null_mut());
            }
        }

        FocusBuffer {
            buffer: tree
        }
    }

    fn first_acceptor_node<'a>(&'a self) -> Option<&'a FocusNode> {
        self.buffer.iter().skip_while(|&a| !a.is_acceptor).next()
    }

    pub fn first_acceptor(&self) -> FocusedElement {
        match self.first_acceptor_node() {
            Some(node) => {
                FocusedElement {
                    focus_node: self.node_as_index(node) as isize,
                    cursor: Cursor::new(node),
                }
            }
            None => {
                FocusedElement {
                    focus_node: -1,
                    cursor: Cursor::default(),
                }
            }
        }
    }

    pub fn global_index(&self, el: &FocusedElement) -> Option<usize> {
        if el.focus_node >= 0 {
            Some(self.node_as_global_index(self.get(el.focus_node as usize).unwrap()) as usize)
        } else {
            None
        }
    }

    pub fn focus_up(&self, previous: &FocusedElement) -> Option<FocusedElement> {
        self.focus_any(previous, direction::focus_up, Axis::Y)
    }

    pub fn focus_down(&self, previous: &FocusedElement) -> Option<FocusedElement> {
        self.focus_any(previous, direction::focus_down, Axis::Y)
    }

    pub fn focus_right(&self, previous: &FocusedElement) -> Option<FocusedElement> {
        self.focus_any(previous, direction::focus_right, Axis::X)
    }

    pub fn focus_left(&self, previous: &FocusedElement) -> Option<FocusedElement> {
        self.focus_any(previous, direction::focus_left, Axis::X)
    }

    fn focus_any<F>(&self, previous: &FocusedElement, pick_next: F, axis: Axis)
        -> Option<FocusedElement>
        where F: Fn(&FocusNode, Cursor) -> &FocusNode
    {
        if previous.focus_node >= 0 {

            assert!((previous.focus_node as usize) < self.len());

            let node =
                pick_next(self.get(previous.focus_node as usize).unwrap(), previous.cursor);
            let new_index = self.node_as_index(node);

            Some(FocusedElement {
                focus_node: new_index,
                cursor: Cursor::from(previous.cursor, node, axis),
            })
        } else {
            None
        }
    }

    pub fn update_nodes(&mut self, layout_data: &LayoutBuffer) {

        for (&i, focus) in self.buffer.enumerate_lookup_indices_mut().unwrap() {
            // This part is always safe because the initialization step
            // ensure that:
            //       self.layout_data.len() >= self.render_data
            //
            // See RenderBuffer#update_nodes
            let boxi = unsafe { layout_data.get_unchecked(i) };
            let ref rec = boxi.dim().content;
            focus.bounds = Rect {
                x: rec.x,
                y: rec.y,
                width: rec.width + boxi.dim().margin.left + boxi.dim().border.left
                    + boxi.dim().margin.right + boxi.dim().border.right,
                height: rec.height + boxi.dim().margin.top + boxi.dim().border.top
                    + boxi.dim().margin.bottom + boxi.dim().border.bottom,
            }
        }

        // Resolve line numbers
        for node in self.buffer.tree_iter_mut()  {
            resolve_line_numbers(node);
        }
    }
}

fn converter(tagged_node: &TaggedNode) -> Option<FocusAcceptor> {
    if tagged_node.is_acceptor || tagged_node.has_children_acceptors {
        Some(FocusAcceptor::new(tagged_node))
    } else {
        None
    }
}

unsafe fn resolve_parent(focus_node: &mut FocusNode, parent: *const FocusNode) {

    focus_node.parent = parent;

    for child in focus_node.children_mut() {
        resolve_parent(child, focus_node);
    }
}

fn resolve_line_numbers(focus_node: &mut FocusNode) {

    let mut current_line_number = 0;
    let mut current_y = focus_node.children()
        .next()
        .map(|c| c.bounds.y)
        .unwrap_or(f32::NEG_INFINITY);

    for child in focus_node.children_mut() {
        // The child `y` property definition shouldn't change
        // to have this working.
        // With the circle layout this might change.

        if child.bounds.y > current_y {
            current_y = child.bounds.y;
            current_line_number += 1;
        }

        child.line_number = current_line_number;

        resolve_line_numbers(child);
    }

}
