use focus::FocusNode;
use util::ref_eq;
use super::find_parent_or_neighbour;

/// This function returns the next node on the right.
///
/// You can then simply access the original node
/// index by doing:
///
/// ```ignore
///     let to = focus::focus_right(from);
///     let index = focus_buffer.original_tree_index(to);
///
///     // ...
/// ```
///
pub fn focus_right<'a>(from: &'a FocusNode) -> &'a FocusNode {
    assert_eq!(from.is_acceptor, true);
    find_parent_or_neighbour(from, from, &from.bounds, find_right_neighbour)
}


fn find_right_neighbour<'a>(parent: &'a FocusNode, from: &'a FocusNode)
    -> Option<&'a FocusNode>
{
    let mut next_child = false;

    // Find our right neighbour
    for child in parent.children() {

        if next_child {
            return Some(child)
        }

        if ref_eq(child, from) {
            next_child = true;
        }
    }

    None
}
