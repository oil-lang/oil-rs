use std::f32;
use std::ptr;
use std::mem;
use std::ops::{Index, Deref};

use util::flat_tree::{FlatTree, TreeNode};
use layout::{LayoutBuffer, Rect};
use style::StyledNode;
use self::tagged_tree::TaggedNode;
use std::default::Default;

mod tagged_tree;
mod direction;

// Reexports
pub use self::direction::focus_up;
pub use self::direction::focus_down;
pub use self::direction::focus_left;
pub use self::direction::focus_right;

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
    // TODO: this should be part of
    //       FlatTree. It is an implementation
    //       details of FlatTree.
    lookup_indices: Box<[usize]>,
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

    pub fn new(root: &StyledNode) -> FocusBuffer {

        let tagged_tree = TaggedNode::new(root);

        let (mut tree, lookup_table) =
            FlatTree::new_with_lookup_table(&tagged_tree, 10, converter);

        // Resolve parents
        for node in tree.tree_iter_mut() {
            unsafe {
                resolve_parent(node, ptr::null_mut());
            }
        }

        FocusBuffer {
            buffer: tree,
            lookup_indices: lookup_table,
        }
    }

    pub fn first_acceptor<'a>(&'a self) -> Option<&'a FocusNode> {
        self.buffer.iter().skip_while(|&a| !a.is_acceptor).next()
    }

    pub fn first_acceptor_index<'a>(&'a self) -> isize {
        self.first_acceptor().map_or(-1, |n| self.buffer.node_as_index(n) as isize)
    }

    pub fn update_nodes(&mut self, layout_data: &LayoutBuffer) {

        for (focus, &i) in self.buffer.iter_mut().zip(self.lookup_indices.iter()) {
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
    let mut current_y = f32::NAN;

    for child in focus_node.children_mut() {
        // The child `y` property definition shouldn't change
        // to have this working.
        // With the circle layout this might change.
        if child.bounds.y > current_y && !current_y.is_nan() {
            current_y = child.bounds.y;
            current_line_number += 1;
        }

        child.line_number = current_line_number;
    }
}
