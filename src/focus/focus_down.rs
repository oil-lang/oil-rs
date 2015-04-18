use super::FocusNode;

/// This function returns the next node on the bottom.
///
/// You can then simply access the original node
/// index by doing:
///
/// ```ignore
///     let to = focus::focusDown(from);
///     let index = focus_buffer.original_tree_index(to);
///
///     // ...
/// ```
///
pub fn focusDown(from: &FocusNode) -> &FocusNode {
    unimplemented!();
}
