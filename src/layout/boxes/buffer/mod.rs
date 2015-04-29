use util::flat_tree::FlatTree;
use util::flat_tree::TreeNode;
use std::ops::Deref;
use super::LayoutBox;
use style::StyledNode;



mod repeat_node;
mod simple_node;




pub struct LayoutBuffer(FlatTree<LayoutBox>);
pub type LayoutNode = TreeNode<LayoutBox>;


impl Deref for LayoutBuffer {
    type Target = [LayoutNode];

    fn deref<'a>(&'a self) -> &'a [LayoutNode] {
        self.0.deref()
    }
}

impl LayoutBuffer {

    pub fn new(style_tree: &StyledNode) -> LayoutBuffer {

        let size = style_tree.tree_size();

        LayoutBuffer(FlatTree::new(style_tree, size, converter))
    }

    pub fn compute_layout(&mut self, max_width: f32, max_height: f32) {

        // First pass: compute default width
        for root in self.0.tree_iter_mut() {
            compute_layout_defaut_width(root, max_width);
        }

        // Second pass: compute auto margins and width auto
        for root in self.0.tree_iter_mut() {
            compute_layout_auto_width(root, max_width);
        }

        // Third pass: layout children and compute their height
        for root in self.0.tree_iter_mut() {
            compute_layout_height_and_position(root, max_height);
        }
    }
}

fn converter(node: &StyledNode) -> Option<LayoutBox> {
    Some(LayoutBox::new(node))
}

/// This function compute the width for this node
/// and return the space it would have eaten if it had more space
/// than the one given.
///
/// This can appear when this node has a child with a fixed width.
fn compute_layout_defaut_width(this: &mut LayoutNode, space_available_for_self: f32) -> f32
{
    // Compute the extra part to remove
    let o = this.dim.padding.left
        + this.dim.padding.right
        + this.dim.margin.left
        + this.dim.margin.right
        + this.dim.border.left
        + this.dim.border.right;

    // Compute the space available for each line
    let space_available = if this.flags.has_width_fixed() {
        this.dim.content.width
    } else {
        space_available_for_self - o
    };

    // Iterating variables
    let mut max = 0f32;
    let mut sum = 0f32;
    let mut line_space_available = space_available;

    // Scope to reduce iter lifetime.
    {
        let mut iter = this.children_mut();
        let mut option_next = iter.next();
        let mut next_child = false;

        loop {

            // This line is confusing...
            // child has the type &mut &mut LayoutBox (one additional indirection)
            // but in release both child, option_next and iter are optimized out.
            // So I guess I shouldn't worry about that, or not ?
            if let Some(ref mut child) = option_next {

                // Recursive call: eat the space given
                let space_eaten = compute_layout_defaut_width(child, line_space_available);

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

    // Assign width for this.
    if !this.flags.has_width_fixed() {
        this.dim.content.width = if this.flags.has_width_expand() {
            space_available
        } else {
            max.min(space_available)
        }
    };

    // Compute the free space for margin in expand mode:
    let s = space_available - this.dim.content.width;

    // We can also compute the margins (left/right) if they're auto:
    match (this.flags.has_margin_right_expand(), this.flags.has_margin_left_expand()) {
        (true, true) => {
            this.dim.margin.left  = s / 2f32;
            this.dim.margin.right = s / 2f32;
        }
        (true, false) => {
            this.dim.margin.left = s;
        }
        (false, true) => {
            this.dim.margin.right = s;
        }
        _ => ()
    }

    if this.flags.has_width_fixed() {
        this.dim.content.width + o
    } else {
        max + o
    }
}

/// This function performs a tree traversal to compute the auto values
/// on nodes in the tree.
/// It should be called after compute_layout_default_width
fn compute_layout_auto_width(this: &mut LayoutNode, space_available: f32)
{
    for child in this.children_mut() {

        compute_layout_auto_width(child, this.dim.content.width);
    }

    // Resolve auto width for this.
    if this.flags.has_width_auto() {
        let o = this.dim.padding.left
            + this.dim.padding.right
            + this.dim.margin.left
            + this.dim.margin.right
            + this.dim.border.left
            + this.dim.border.right;

        this.dim.content.width = space_available - o;
    }

    // Compute the free space for margin in auto mode:
    let s = space_available - this.dim.content.width;

    // We can also compute the margins (left/right) if they're auto:
    match (this.flags.has_margin_right_auto(), this.flags.has_margin_left_auto()) {
        (true, true) => {
            this.dim.margin.left  = s / 2f32;
            this.dim.margin.right = s / 2f32;
        }
        (true, false) => {
            this.dim.margin.left = s;
        }
        (false, true) => {
            this.dim.margin.right = s;
        }
        _ => ()
    }
}

//
// TODO: FIXME Text nodes must have their size precomputed somehow
//
// PRECONDITONS: compute_width has been called
//
fn compute_layout_height_and_position(this: &mut LayoutNode, max_height: f32)
{

    // At this point we don't know this.dim.height / this.dim.width
    // positions
    let mut x = this.dim.content.x
        + this.dim.padding.left
        + this.dim.margin.left
        + this.dim.border.left;
    let mut y = this.dim.content.y
        + this.dim.padding.top
        + this.dim.margin.top
        + this.dim.border.top;

    // Equivalent rule for child max height:
    let child_max_height = if this.flags.has_height_fixed() {

        this.dim.content.height
    } else {

        max_height
        - this.dim.padding.bottom - this.dim.padding.top
        - this.dim.border.bottom  - this.dim.border.top
        - this.dim.margin.bottom  - this.dim.margin.top
    };

    // Current line width allow to track the layout progress
    // in the x direction while height allow to track the y direction
    let mut current_line_width  = 0f32;
    let mut current_line_height = 0f32;
    let mut current_height_left = child_max_height;
    let mut accumulated_line_height = 0f32;

    // Reduced scope for stack (borrowck problem otherwise)
    {
        // Used for margin (top/bottom)
        let mut stack: Vec<&mut LayoutNode> = Vec::with_capacity(4);

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

        for child in this.children_mut() {

            let child_is_auto = child.flags.is_auto();

            // Line return ?
            if child.dim.content.width + current_line_width > this.dim.content.width {
                let ref d = this.dim;
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

            compute_layout_height_and_position(child, current_height_left);


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
                let ref d = this.dim;
                line_return!(
                    stack,
                    current_line_height,
                    current_line_width,
                    current_height_left,
                    accumulated_line_height,
                    d);
            }
        }
    }

    // Finally: the height !
    this.dim.content.height =
        this.dim.content.height.max(child_max_height.min(accumulated_line_height));
}
