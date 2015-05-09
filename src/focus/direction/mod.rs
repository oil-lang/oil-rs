use num::traits::ToPrimitive;

use focus::FocusNode;
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


fn distance(a: &Cursor, b:&Cursor) -> f32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}


mod down;
mod up;
mod right;
mod left;

// ======================================== //
//             Left/Right Logic             //
// ======================================== //

fn find_matching_child_x<'a>(
    from: &'a FocusNode,
    parent: &'a FocusNode,
    cursor: &Cursor,
    weight: f32)
    -> &'a FocusNode
{

    // Pick the best child
    let mut factor = 0f32;
    let sign = weight.signum();
    let res = parent.children()
        .filter(|n| (n.bounds.x - cursor.x) * sign > 0.0 )
        .map(|n| {
            factor += 1f32;
            (distance(&Cursor::new(n), cursor) * (1.0 + weight / factor), n)
        })
        .min_by(|&(w, _)| F32Ord(w));

    if let Some((_, node)) = res {
        if node.is_acceptor {
            node
        } else {
            find_matching_child_x(from, node, cursor, weight)
        }
    } else {
        from
    }
}

fn find_parent_or_neighbour<'a, F>(
    from: &'a FocusNode,
    current: &'a FocusNode,
    cursor: &Cursor,
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
                        return find_matching_child_x(from, child, cursor, weight);
                    }
                }

            }
            // Not found ? -> Look for parent.
            find_parent_or_neighbour(from, parent, cursor, weight, neighbour_finder)
        }
        // No parent ?
        None => from
    }
}

// ======================================== //
//                Up/Down Logic             //
// ======================================== //


fn find_matching_child_y<'a>(
    from: &'a FocusNode,
    parent: &'a FocusNode,
    cursor: &Cursor,
    sign: f32)
    -> &'a FocusNode
{

    // Pick the best child
    let mut factor = 0f32;
    let res = parent.children()
        .filter(|n| (n.bounds.y - cursor.y) * sign > 0.0 )
        .map(|n| {
            factor += 1f32;
            (distance(&Cursor::new(n), cursor) * (1.0 + 0.1 / factor), n)
        })
        .min_by(|&(w, _)| F32Ord(w));

    if let Some((_, node)) = res {
        if node.is_acceptor {
            node
        } else {
            find_matching_child_y(from, node, cursor, sign)
        }
    } else {
        from
    }
}

fn find_neighbor<'a>(
    from: &'a FocusNode,
    current_node: &'a FocusNode,
    cursor: &Cursor,
    offset: isize)
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
                    find_matching_child_y(from, node, cursor, offset.signum().to_f32().unwrap())
                }
            } else {
                find_neighbor(from, parent, cursor, offset)
            }
        }
        // No parent ?
        None => from
    }
}
