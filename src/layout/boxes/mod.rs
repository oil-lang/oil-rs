
use std::num::Float;

use super::dim::{self, DimFlags};
use super::{Dimensions, EdgeSizes, Rect};
use style::{StyledNode, PropertyName};

// Reexport iterator for buffer
pub use self::iter::{LayoutBoxIterMut, LayoutBoxIter};

mod iter;

// The layout box kids are unsorted (defined by the markup)
// except the one declared with absolute positioning. They will
// end up at the end sorted by z-index.
pub struct LayoutBox {
    dim: Dimensions,
    // Stores auto/fixed behaviors
    flags: DimFlags,
    next_sibling: isize,
}

// ======================================== //
//                 INTERFACE                //
// ======================================== //

impl LayoutBox {

    pub unsafe fn new(node: &StyledNode, next_sibling: isize) -> LayoutBox {
        let mut flags = DimFlags::empty();

        if node.is_property_auto(PropertyName::MARGIN_LEFT) {
            flags = flags | dim::MARGIN_LEFT_AUTO;
        }

        if node.is_property_auto(PropertyName::MARGIN_RIGHT) {
            flags = flags | dim::MARGIN_RIGHT_AUTO;
        }

        if node.is_property_auto(PropertyName::MARGIN_TOP) {
            flags = flags | dim::MARGIN_TOP_AUTO;
        }

        if node.is_property_auto(PropertyName::MARGIN_BOTTOM) {
            flags = flags | dim::MARGIN_BOT_AUTO;
        }

        if node.is_property_auto(PropertyName::WIDTH) {
            flags = flags | dim::WIDTH_AUTO;
        }

        if node.is_property_expand(PropertyName::WIDTH) {
            flags = flags | dim::WIDTH_EXPAND;
        }

        if node.is_property_expand(PropertyName::MARGIN_LEFT) {
            flags = flags | dim::MARGIN_LEFT_EXPAND;
        }

        if node.is_property_expand(PropertyName::MARGIN_RIGHT) {
            flags = flags | dim::MARGIN_RIGHT_EXPAND;
        }

        let padding_left = node.size_prop(PropertyName::PADDING_LEFT);
        let padding_right = node.size_prop(PropertyName::PADDING_RIGHT);
        let padding_top = node.size_prop(PropertyName::PADDING_TOP);
        let padding_bottom = node.size_prop(PropertyName::PADDING_BOTTOM);

        let margin_left = node.size_prop(PropertyName::MARGIN_LEFT);
        let margin_right = node.size_prop(PropertyName::MARGIN_RIGHT);
        let margin_top = node.size_prop(PropertyName::MARGIN_TOP);
        let margin_bottom = node.size_prop(PropertyName::MARGIN_BOTTOM);

        let border_left = node.size_prop(PropertyName::BORDER_LEFT);
        let border_right = node.size_prop(PropertyName::BORDER_RIGHT);
        let border_top = node.size_prop(PropertyName::BORDER_TOP);
        let border_bottom = node.size_prop(PropertyName::BORDER_BOTTOM);

        let width = match node.size_prop_as_opt(PropertyName::WIDTH) {
            Some(w) => {
                flags = flags | dim::WIDTH_FIXED;
                w
            }
            None => 0f32
        };

        let height = match node.size_prop_as_opt(PropertyName::HEIGHT) {
            Some(h) => {
                flags = flags | dim::HEIGHT_FIXED;
                h
            }
            None => 0f32
        };

        // TODO: Missing bit for left / right / top / bottom
        //       We also need at some point the relative information

        LayoutBox {
            dim: Dimensions {
                content: Rect {
                    x: 0f32,
                    y: 0f32,
                    width: width,
                    height: height,
                },
                padding: EdgeSizes {
                    left: padding_left,
                    right: padding_right,
                    top: padding_top,
                    bottom: padding_bottom,
                },
                border: EdgeSizes {
                    left: border_left,
                    right: border_right,
                    top: border_top,
                    bottom: border_bottom
                },
                margin: EdgeSizes {
                    left: margin_left,
                    right: margin_right,
                    top: margin_top,
                    bottom: margin_bottom
                }
            },
            flags: flags,
            next_sibling: next_sibling,
        }
    }

    #[inline]
    pub unsafe fn set_next_sibling(&mut self, next_sibling: isize) {
        self.next_sibling = next_sibling;
    }

    pub fn children<'a>(&'a self) -> LayoutBoxIter {
        if self.next_sibling > 1 || self.next_sibling == -1 {
            unsafe { LayoutBoxIter::new_with_firstchild(self) }
        } else {
            LayoutBoxIter::new_empty()
        }
    }

    pub fn children_mut<'a>(&'a self) -> LayoutBoxIterMut {
        if self.next_sibling > 1 || self.next_sibling == -1 {
            unsafe { LayoutBoxIterMut::new_with_firstchild(self) }
        } else {
            LayoutBoxIterMut::new_empty()
        }
    }

    #[inline]
    pub fn dim(&self) -> Dimensions {
        self.dim
    }

    /// This function compute the width for this node
    /// and return the space it would have eaten if it had more space
    /// than the one given.
    ///
    /// This can appear when this node has a child with a fixed width.
    pub fn compute_layout_defaut_width(&mut self, space_available_for_self: f32) -> f32
    {
        // Compute the extra part to remove
        let o = self.dim.padding.left
            + self.dim.padding.right
            + self.dim.margin.left
            + self.dim.margin.right
            + self.dim.border.left
            + self.dim.border.right;

        // Compute the space available for each line
        let space_available = if self.flags.has_width_fixed() {
            self.dim.content.width
        } else {
            space_available_for_self - o
        };

        // Iterating variables
        let mut max = 0f32;
        let mut sum = 0f32;
        let mut line_space_available = space_available;

        // Scope to reduce iter lifetime.
        {
            let mut iter = self.children_mut();
            let mut option_next = iter.next();
            let mut next_child = false;

            loop {

                // This line is confusing...
                // child has the type &mut &mut LayoutBox (one additional indirection)
                // but in release both child, option_next and iter are optimized out.
                // So I guess I shouldn't worry about that, or not ?
                if let Some(ref mut child) = option_next {

                    // Recursive call: eat the space given
                    let space_eaten = child.compute_layout_defaut_width(line_space_available);

                    // If the child has not eaten more than given
                    // then we just reduce the space available for the next child
                    if line_space_available >= space_eaten {

                        line_space_available -= space_eaten;
                        sum += space_eaten;

                        // Should we increase the max line size ?
                        if sum > max {
                            max = sum;
                        }

                        // If the child is auto then we simply restart with a new line
                        if child.flags.is_auto() {
                            sum = 0f32;
                            line_space_available = space_available;
                        }

                        // We'll go see for the next child now.
                        next_child = true;

                    // Otherwise, we have to put the child on a new line
                    // except if the child was alone on its line
                    } else {

                        // Am I alone ?
                        if line_space_available == space_available {

                            // This might be surprising, but here we want to
                            // transmit back to our parent the child constraint.
                            if sum > max {
                                max = sum;
                            }

                            // Then start a new line:
                            sum = 0f32;

                            next_child = true;

                        // If I wasn't alone, as I will be on a new line,
                        // the recursion will be done again. This do have an overhead.
                        }

                    }

                } else {
                    break;
                }

                if next_child {

                    // Go take care of the next child:
                    option_next = iter.next();

                    next_child = false;
                }
            }
        }

        // Assign width for self.
        if !self.flags.has_width_fixed() {
            self.dim.content.width = if self.flags.has_width_expand() {
                space_available
            } else {
                max.min(space_available)
            }
        };

        // Compute the free space for margin in expand mode:
        let s = space_available - self.dim.content.width;

        // We can also compute the margins (left/right) if they're auto:
        match (self.flags.has_margin_right_expand(), self.flags.has_margin_left_expand()) {
            (true, true) => {
                self.dim.margin.left  = s / 2f32;
                self.dim.margin.right = s / 2f32;
            }
            (true, false) => {
                self.dim.margin.left = s;
            }
            (false, true) => {
                self.dim.margin.right = s;
            }
            _ => ()
        }

        if self.flags.has_width_fixed() {
            self.dim.content.width + o
        } else {
            max + o
        }
    }

    /// This function performs a tree traversal to compute the auto values
    /// on nodes in the tree.
    /// It should be called after compute_layout_default_width
    pub fn compute_layout_auto_width(&mut self, space_available: f32)
    {
        for child in self.children_mut() {

            child.compute_layout_auto_width(self.dim.content.width);
        }

        // Resolve auto width for self.
        if self.flags.has_width_auto() {
            let o = self.dim.padding.left
                + self.dim.padding.right
                + self.dim.margin.left
                + self.dim.margin.right
                + self.dim.border.left
                + self.dim.border.right;

            self.dim.content.width = space_available - o;
        }

        // Compute the free space for margin in auto mode:
        let s = space_available - self.dim.content.width;

        // We can also compute the margins (left/right) if they're auto:
        match (self.flags.has_margin_right_auto(), self.flags.has_margin_left_auto()) {
            (true, true) => {
                self.dim.margin.left  = s / 2f32;
                self.dim.margin.right = s / 2f32;
            }
            (true, false) => {
                self.dim.margin.left = s;
            }
            (false, true) => {
                self.dim.margin.right = s;
            }
            _ => ()
        }
    }

    //
    // TODO: FIXME Text nodes must have their size precomputed somehow
    //
    // PRECONDITONS: compute_width has been called
    //
    pub fn compute_layout_height_and_position(
        &mut self,
        max_height: f32)
    {

        // At this point we don't know self.dim.height / self.dim.width
        // positions
        let mut x = self.dim.content.x
            + self.dim.padding.left
            + self.dim.margin.left
            + self.dim.border.left;
        let mut y = self.dim.content.y
            + self.dim.padding.top
            + self.dim.margin.top
            + self.dim.border.top;

        // Equivalent rule for child max height:
        let child_max_height = if self.flags.has_height_fixed() {

            self.dim.content.height
        } else {

            max_height
            - self.dim.padding.bottom - self.dim.padding.top
            - self.dim.border.bottom  - self.dim.border.top
            - self.dim.margin.bottom  - self.dim.margin.top
        };

        // Current line width allow to track the layout progress
        // in the x direction while height allow to track the y direction
        let mut current_line_width  = 0f32;
        let mut current_line_height = 0f32;
        let mut current_height_left = child_max_height;
        let mut accumulated_line_height = 0f32;

        // Used for margin (top/bottom)
        let mut stack: Vec<&mut LayoutBox> = Vec::with_capacity(4);

        macro_rules! line_return {

            ($stack:ident,
             $current_line_height:ident,
             $current_line_width:ident,
             $current_height_left:ident,
             $accumulated_line_height:ident,
             $d:ident) => ({

                while let Some(ref mut c) = $stack.pop() {

                    let s = $current_line_height - c.dim.content.height
                        - c.dim.padding.top - c.dim.padding.bottom
                        - c.dim.border.top  - c.dim.border.bottom
                        - c.dim.margin.top  - c.dim.margin.bottom;

                    // We compute the margin top / bottom
                    match (c.flags.has_margin_top_auto(), c.flags.has_margin_bottom_auto()) {
                        (true, true) => {
                            c.dim.margin.top    = s / 2f32;
                            c.dim.margin.bottom = s / 2f32;
                        }
                        (true, false) => {
                            c.dim.margin.top   = s;
                        }
                        (false, true) => {
                            c.dim.margin.bottom  = s;
                        }
                        _ => ()
                    }
                }

                x = $d.content.x + $d.padding.left + $d.border.left + $d.margin.left;
                y += $current_line_height;
                $accumulated_line_height += $current_line_height;
                $current_height_left     -= $current_line_height;
                $current_line_width   = 0f32;
                $current_line_height  = 0f32;
            });
        }

        for child in self.children_mut() {

            let child_is_auto = child.flags.is_auto();

            // Line return ?
            if child.dim.content.width + current_line_width > self.dim.content.width {
                let ref d = self.dim;
                line_return!(
                    stack,
                    current_line_height,
                    current_line_width,
                    current_height_left,
                    accumulated_line_height,
                    d);
            }

            child.dim.content.x = x;
            child.dim.content.y = y;

            // Update the x position:
            let child_total_width = child.dim.content.width
                + child.dim.margin.left
                + child.dim.margin.right
                + child.dim.border.left
                + child.dim.border.right;

            x += child_total_width;
            current_line_width += child_total_width;

            child.compute_layout_height_and_position(current_height_left);


            // Note: at this point child.margin (top, right) are either fixed
            // or zero (if they were auto). They will be computed in a later pass.
            current_line_height = current_line_height.max(child.dim.content.height
                + child.dim.margin.top
                + child.dim.margin.bottom
                + child.dim.border.top
                + child.dim.border.bottom
            );

            if child.flags.has_margin_top_or_bot_auto() {
                stack.push(child);
            }

            if child_is_auto {
                let ref d = self.dim;
                line_return!(
                    stack,
                    current_line_height,
                    current_line_width,
                    current_height_left,
                    accumulated_line_height,
                    d);
            }
        }

        // Finally: the height !
        self.dim.content.height =
            self.dim.content.height.max(child_max_height.min(accumulated_line_height));
    }
}
