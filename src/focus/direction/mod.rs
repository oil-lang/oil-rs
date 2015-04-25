use focus::FocusNode;
use layout::Rect;
use util::F32Ord;
use util::ref_eq;

pub use self::up::focus_up;
pub use self::down::focus_down;
pub use self::left::focus_left;
pub use self::right::focus_right;

mod down;
mod up;
mod right;
mod left;


fn find_matching_child<'a>(parent: &'a FocusNode, bounds: &Rect) -> &'a FocusNode {

    // Pick the best child
    let res = parent.children().map(|n| {
        (n.bounds.intersects(bounds), n)
    }).max_by(|&(w, _)| F32Ord(w));

    if let Some((_, node)) = res {
        if node.is_acceptor {
            node
        } else {
            find_matching_child(node, bounds)
        }
    } else {
        // The parent can't be non-acceptor and have zero children.
        panic!("This parent does not have any children ? Bug found !");
    }
}

fn find_parent_or_neighbour<'a, F>(
    from: &'a FocusNode,
    current: &'a FocusNode,
    bounds: &Rect,
    neighbour_finder: F) -> &'a FocusNode
    where F: Fn(&'a FocusNode, &'a FocusNode) -> Option<&'a FocusNode>
{

    // Look for parent
    match current.parent() {

        Some(parent) => {

            if let Some(child) = neighbour_finder(parent, from) {

                // Is it on the same line ?
                if child.line_number == current.line_number && !ref_eq(child, from) {
                    // Did we found a node acceptor ?
                    if child.is_acceptor {
                        // Then we're done.
                        return child;

                    } else {
                        // Ok we switch to a different reasoning now:
                        return find_matching_child(child, bounds)
                    }
                }
            }

            // Not found ? -> Look for parent.
            find_parent_or_neighbour(from, parent, bounds, neighbour_finder)
        }
        // No parent ?
        None => from
    }
}
