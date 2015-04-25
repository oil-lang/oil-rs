use focus::FocusNode;

/// This function returns the next node on the bottom.
///
/// You can then simply access the original node
/// index by doing:
///
/// ```ignore
///     let to = focus::focus_down(from);
///     let index = focus_buffer.original_tree_index(to);
///
///     // ...
/// ```
///
pub fn focus_down(from: &FocusNode) -> &FocusNode {
    unimplemented!();
}
