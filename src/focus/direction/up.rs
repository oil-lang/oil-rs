use focus::FocusNode;
use super::Cursor;
use super::find_neighbor;

/// This function returns the next node on the top.
///
/// You can then simply access the original node
/// index by doing:
///
/// ```ignore
///     let to = focus::focus_up(from);
///     let index = focus_buffer.node_as_global_index(to);
///
///     // ...
/// ```
///
pub fn focus_up<'a>(from: &'a FocusNode, cursor: Cursor) -> &'a FocusNode {
    find_neighbor(from, from, -1)
}
