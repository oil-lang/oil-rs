
use std::num::Float;
use std::default::Default;

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
    max_width: f32,
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
            max_width: 0f32,
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

    pub fn compute_max_width(&mut self)
    {
        let o = self.dim.padding.left
            + self.dim.padding.right
            + self.dim.margin.left
            + self.dim.margin.right
            + self.dim.border.left
            + self.dim.border.right;

        // Compute max width by using the max width length of a line
        let mut max = 0f32;
        let mut sum = 0f32;
        for child in self.children_mut() {
            // Compute child max width
            child.compute_max_width();
            sum += child.max_width;

            if sum > max {
                max = sum;
            }
            if child.flags.is_auto() {
                sum = 0f32;
            }
        }

        // Assign max width for self.
        self.max_width = if self.flags.has_width_fixed() {
            self.dim.content.width + o
        } else {
            sum + o
        };
    }

    fn get_bigger_line_size(&self, max_width: f32) -> f32
    {

        let mut max = 0f32;
        let mut current_line_width  = 0f32;

        macro_rules! line_return {
            ($max:ident, $current_line_width:ident) => ({
                $max = $max.max($current_line_width);
                $current_line_width  = 0f32;
            });
        }

        for child in self.children() {
            let child_full_width = child.max_width;

            // Line return ?
            if child_full_width + current_line_width > max_width {
                line_return!(max, current_line_width);
            }

            current_line_width += child_full_width;

            if child.flags.is_auto() {
                line_return!(max, current_line_width);
            }
        }

        max
    }


    //
    // TODO: FIXME Text nodes must have their size precomputed
    //
    // PRECONDITONS: Everything initialized using PropertyName.
    //
    pub fn compute_layout(
        &mut self,
        max_width: f32,
        max_height: f32)
    {
        // We can compute directly the self.dim.width
        // and the space for each line
        let max_line_width  = max_width.min(self.get_bigger_line_size(max_width));

        // Syntax sugar
        let ref node = self.flags;

        // At this point we don't know self.dim.height / self.dim.width
        // positions
        let mut x = self.dim.content.x + self.dim.padding.left + self.dim.border.left;
        let mut y = self.dim.content.y + self.dim.padding.top  + self.dim.border.top;

        // Child max width calculation
        let child_max_width = if self.flags.has_width_fixed() {

            // The max width will be the content width.
            self.dim.content.width
        } else {
            // In that case, the width can be immediately computed as:
            self.dim.content.width = max_line_width;

            // The resulting max_width for children is:
            max_width
            - self.dim.padding.right - self.dim.padding.left
            - self.dim.border.right  - self.dim.border.left
            - self.dim.margin.right  - self.dim.margin.left
        };

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
                        - c.dim.padding.right - c.dim.padding.left
                        - c.dim.border.right  - c.dim.border.left
                        - c.dim.margin.right  - c.dim.margin.left;
                    // We compute the margin top / bottom
                    match (c.flags.has_margin_top_auto(), c.flags.has_margin_bottom_auto()) {
                        (true, true) => {
                            c.dim.margin.top    = s / 2f32;
                            c.dim.margin.bottom = s / 2f32;
                        }
                        (true, false) => {
                            c.dim.margin.left   = s;
                        }
                        (false, true) => {
                            c.dim.margin.right  = s;
                        }
                        _ => ()
                    }
                }

                x = $d.content.x + $d.padding.left + $d.border.left;
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
            if child.max_width + current_line_width > child_max_width {
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

            child.compute_layout(child_max_width - current_line_width,
                                 current_height_left);

            current_line_width += child.dim.content.width
                + child.dim.margin.left
                + child.dim.margin.right
                + child.dim.border.left
                + child.dim.border.right;

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


        // Now we do know self.dim.height / self.dim.width
        // We just do some adjustement
        if node.has_width_auto() {
            self.dim.content.width = child_max_width;
        }

        self.dim.content.height =
            self.dim.content.height.max(child_max_height.min(accumulated_line_height));

        // Compute the free space for margin in auto mode:
        let s = child_max_width - self.dim.content.width;

        // We can also compute the margins (left/right) if they're auto:
        match (node.has_margin_right_auto(), node.has_margin_left_auto()) {
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
}
