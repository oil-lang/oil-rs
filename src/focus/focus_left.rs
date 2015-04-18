use super::FocusNode;

/// This function returns the next node on the left.
///
/// You can then simply access the original node
/// index by doing:
///
/// ```ignore
///     let to = focus::focusLeft(from);
///     let index = focus_buffer.original_tree_index(to);
///
///     // ...
/// ```
///
pub fn focusLeft(from: &FocusNode) -> &FocusNode {
    unimplemented!();
}
