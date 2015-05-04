use focus::FocusNode;
use util::ref_eq;
use super::find_parent_or_neighbour;

/// This function returns the next node on the left.
///
/// You can then simply access the original node
/// index by doing:
///
/// ```ignore
///     let to = focus::focus_left(from);
///     let index = focus_buffer.original_tree_index(to);
///
///     // ...
/// ```
///
pub fn focus_left(from: &FocusNode) -> &FocusNode {
    assert_eq!(from.is_acceptor, true);
    find_parent_or_neighbour(from, from, &from.bounds, -0.1, find_left_neighbour)
}

fn find_left_neighbour<'a>(parent: &'a FocusNode, from: &'a FocusNode)
    -> Option<&'a FocusNode>
{

    let mut prev_child = None;

    // Find our left neighbour
    for child in parent.children() {

        if ref_eq(child, from) {
            return prev_child;
        }

        prev_child = Some(child);
    }

    None
}
