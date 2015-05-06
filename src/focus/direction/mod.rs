use focus::FocusNode;
use layout::Rect;
use util::F32Ord;
use util::ref_eq;

pub use self::up::focus_up;
pub use self::down::focus_down;
pub use self::left::focus_left;
pub use self::right::focus_right;


#[derive(Copy, Clone, Default)]
pub struct Cursor {
    x: f32,
    y: f32
}

pub enum Axis {
    X,
    Y
}

impl Cursor {

    pub fn new(node: &FocusNode) -> Cursor {
        Cursor {
            x: node.bounds.x + node.bounds.width / 2.0,
            y: node.bounds.y + node.bounds.height / 2.0,
        }
    }

    pub fn from(previous: Cursor, node: &FocusNode, axis: Axis) -> Cursor {
        match axis {
            Axis::X => Cursor {
                x: node.bounds.x + node.bounds.width / 2.0,
                y: previous.y,
            },
            Axis::Y => Cursor {
                x: previous.x,
                y: node.bounds.y + node.bounds.height / 2.0,
            },
        }
    }
}


mod down;
mod up;
mod right;
mod left;

// ======================================== //
//             Left/Right Logic             //
// ======================================== //

fn find_matching_child_x<'a>(parent: &'a FocusNode, bounds: &Rect, weight: f32) -> &'a FocusNode {

    // Pick the best child
    let mut factor = 0f32;
    let res = parent.children().map(|n| {
        factor += 1f32;
        (n.bounds.intersects_y(bounds) * (1.0 + weight / factor), n)
    }).max_by(|&(w, _)| F32Ord(w));

    if let Some((_, node)) = res {
        if node.is_acceptor {
            node
        } else {
            find_matching_child_x(node, bounds, weight)
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
    weight: f32,
    neighbour_finder: F) -> &'a FocusNode
    where F: Fn(&'a FocusNode, &'a FocusNode) -> Option<&'a FocusNode>
{

    // Look for parent
    match current.parent() {

        Some(parent) => {

            if let Some(child) = neighbour_finder(parent, current) {

                // Is it on the same line ?
                if child.line_number == current.line_number && !ref_eq(child, from) {
                    // Did we found a node acceptor ?
                    if child.is_acceptor {
                        // Then we're done.
                        return child;

                    } else {
                        // Ok we switch to a different reasoning now:
                        return find_matching_child_x(child, bounds, weight);
                    }
                }

            }
            // Not found ? -> Look for parent.
            find_parent_or_neighbour(from, parent, bounds, weight, neighbour_finder)
        }
        // No parent ?
        None => from
    }
}

// ======================================== //
//                Up/Down Logic             //
// ======================================== //


fn find_matching_child_y<'a>(parent: &'a FocusNode, bounds: &Rect) -> &'a FocusNode {

    // Pick the best child
    let mut factor = 0f32;
    let res = parent.children().map(|n| {
        factor += 1f32;
        (n.bounds.intersects_x(bounds) * (1.0 + 0.1 / factor), n)
    }).max_by(|&(w, _)| F32Ord(w));

    if let Some((_, node)) = res {
        if node.is_acceptor {
            node
        } else {
            find_matching_child_y(node, bounds)
        }
    } else {
        // The parent can't be non-acceptor and have zero children.
        panic!("This parent does not have any children ? Bug found !");
    }
}

fn find_neighbor<'a>(from: &'a FocusNode, current_node: &'a FocusNode, offset: isize)
    -> &'a FocusNode
{
    match current_node.parent() {

        Some(parent) => {
            let mut factor = 0f32;
            let res = parent.children()
                .filter(|n| n.line_number == (current_node.line_number as isize + offset) as usize)
                .map(|n| {
                    factor += 1f32;
                    (n.bounds.intersects_x(&from.bounds) * (1.0 + 0.1 / factor), n)
                })
                .max_by(|&(w, _)| F32Ord(w));

            if let Some((_, node)) = res {
                if node.is_acceptor {
                    node
                } else {
                    find_matching_child_y(node, &from.bounds)
                }
            } else {
                find_neighbor(from, parent, offset)
            }
        }
        // No parent ?
        None => from
    }
}
