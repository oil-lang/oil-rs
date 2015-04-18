use super::FocusNode;
use layout::Rect;
use util::F32Ord;

/// This function returns the next node on the right.
///
/// You can then simply access the original node
/// index by doing:
///
/// ```ignore
///     let to = focus::focusRight(from);
///     let index = focus_buffer.original_tree_index(to);
///
///     // ...
/// ```
///
pub fn focusRight<'a>(from: &'a FocusNode) -> &'a FocusNode {
    assert_eq!(from.is_acceptor, true);
    find_parent_for_right_neighbour(from, from, &from.bounds)
}

fn find_matching_children<'a>(parent: &'a FocusNode, bounds: &Rect) -> &'a FocusNode {

    // Pick the best child
    let res = parent.children().map(|n| {
        (n.bounds.intersects(bounds), n)
    }).max_by(|&(w, _)| F32Ord(w));

    if let Some((_, node)) = res {
        if node.is_acceptor {
            node
        } else {
            find_matching_children(node, bounds)
        }
    } else {
        // The parent can't be non-acceptor and have zero children.
        panic!("This parent does not have any children ? Bug found !");
    }
}

fn find_parent_for_right_neighbour<'a>(from: &'a FocusNode, current: &'a FocusNode, bounds: &Rect) -> &'a FocusNode {

    // Look for parent
    match current.parent() {

        Some(parent) => {

            let mut next_child = false;

            // Find our right neighbour
            for child in parent.children() {
                if next_child == true {
                    // Is it on the same line ?
                    if child.line_number == current.line_number {
                        // Did we found a node acceptor ?
                        if child.is_acceptor {
                            // Then we're done.
                            return child;

                        } else {
                            // Ok we switch to a different reasoning now:
                            return find_matching_children(child, bounds)
                        }
                    }

                    // If it is not we stop iterating.
                    break;
                }
                if child as *const FocusNode == from as *const FocusNode {
                    next_child = true;
                }
            }

            // Not found ? -> Look for parent.
            find_parent_for_right_neighbour(from, parent, bounds)
        }
        // No parent ?
        None => from
    }
}
