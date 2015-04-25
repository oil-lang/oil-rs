use focus::FocusNode;

/// This function returns the next node on the top.
///
/// You can then simply access the original node
/// index by doing:
///
/// ```ignore
///     let to = focus::focus_up(from);
///     let index = focus_buffer.original_tree_index(to);
///
///     // ...
/// ```
///
pub fn focus_up(from: &FocusNode) -> &FocusNode {
    unimplemented!();
}
