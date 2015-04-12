
use super::dim::{self, DimFlags};
use super::{Dimensions, EdgeSizes, Rect};
use style::{StyledNode, PropertyName};

/// Reexport
pub use self::buffer::LayoutBuffer;
pub use self::buffer::LayoutNode;

mod buffer;

// The layout box kids are unsorted (defined by the markup)
// except the one declared with absolute positioning. They will
// end up at the end sorted by z-index.
pub struct LayoutBox {
    dim: Dimensions,
    // Stores auto/fixed behaviors
    flags: DimFlags,
}

// ======================================== //
//                 INTERFACE                //
// ======================================== //

impl LayoutBox {

    #[inline]
    pub fn dim(&self) -> Dimensions {
        self.dim
    }

    fn new(node: &StyledNode) -> LayoutBox {
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
        }
    }
}
