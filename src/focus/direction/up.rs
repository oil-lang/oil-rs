use focus::FocusNode;

/// This function returns the next node on the top.
///
/// You can then simply access the original node
/// index by doing:
///
/// ```ignore
///     let to = focus::focusUp(from);
///     let index = focus_buffer.original_tree_index(to);
///
///     // ...
/// ```
///
pub fn focusUp(from: &FocusNode) -> &FocusNode {
    unimplemented!();
}
