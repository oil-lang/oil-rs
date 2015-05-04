use super::dim::{self, DimFlags};
use super::{Dimensions, EdgeSizes, Rect};
use uil_shared::properties::PropertyName;
use state::StateData;

/// Reexport
pub use self::buffer::LayoutBuffer;
pub use self::buffer::LayoutNode;

mod buffer;

// The layout box kids are unsorted (defined by the markup)
// except the one declared with absolute positioning. They will
// end up at the end sorted by z-index.
#[derive(Default)]
pub struct LayoutBox {
    dim: Dimensions,
    // Stores auto/fixed behaviors
    flags: DimFlags,
}

// TODO:
pub struct LayoutBoxRepeat {
    dim: Dimensions,
    flags: DimFlags,
    // repeater: TemplateSomething,
    // Vec<DataPerTemplateRequired>
}

// ======================================== //
//                 INTERFACE                //
// ======================================== //

impl LayoutBox {

    #[inline]
    pub fn dim(&self) -> Dimensions {
        self.dim
    }

    pub fn update_from_state(&mut self, state: &StateData) {
        let mut flags = DimFlags::empty();

        // Auto states
        if state.has_property_auto(PropertyName::MARGIN) {

            flags = flags | dim::MARGIN_LEFT_AUTO;
            flags = flags | dim::MARGIN_RIGHT_AUTO;
            flags = flags | dim::MARGIN_TOP_AUTO;
            flags = flags | dim::MARGIN_BOT_AUTO;

        } else {

            if state.has_property_auto(PropertyName::MARGIN_LEFT) {
                flags = flags | dim::MARGIN_LEFT_AUTO;
            }

            if state.has_property_auto(PropertyName::MARGIN_RIGHT) {
                flags = flags | dim::MARGIN_RIGHT_AUTO;
            }

            if state.has_property_auto(PropertyName::MARGIN_TOP) {
                flags = flags | dim::MARGIN_TOP_AUTO;
            }

            if state.has_property_auto(PropertyName::MARGIN_BOTTOM) {
                flags = flags | dim::MARGIN_BOT_AUTO;
            }
        }

        // Expands
        if state.has_property_expand(PropertyName::MARGIN) {

            flags = flags | dim::MARGIN_LEFT_EXPAND;
            flags = flags | dim::MARGIN_RIGHT_EXPAND;

        } else {

            if state.has_property_expand(PropertyName::MARGIN_LEFT) {
                flags = flags | dim::MARGIN_LEFT_EXPAND;
            }

            if state.has_property_expand(PropertyName::MARGIN_RIGHT) {
                flags = flags | dim::MARGIN_RIGHT_EXPAND;
            }
        }

        if state.has_property_auto(PropertyName::WIDTH) {
            flags = flags | dim::WIDTH_AUTO;
        }

        if state.has_property_expand(PropertyName::WIDTH) {
            flags = flags | dim::WIDTH_EXPAND;
        }

        // Sizes
        let padding_left = state.size_of_prop(PropertyName::PADDING_LEFT);
        let padding_right = state.size_of_prop(PropertyName::PADDING_RIGHT);
        let padding_top = state.size_of_prop(PropertyName::PADDING_TOP);
        let padding_bottom = state.size_of_prop(PropertyName::PADDING_BOTTOM);

        let margin_left = state.size_of_prop(PropertyName::MARGIN_LEFT);
        let margin_right = state.size_of_prop(PropertyName::MARGIN_RIGHT);
        let margin_top = state.size_of_prop(PropertyName::MARGIN_TOP);
        let margin_bottom = state.size_of_prop(PropertyName::MARGIN_BOTTOM);

        let border_left = state.size_of_prop(PropertyName::BORDER_LEFT);
        let border_right = state.size_of_prop(PropertyName::BORDER_RIGHT);
        let border_top = state.size_of_prop(PropertyName::BORDER_TOP);
        let border_bottom = state.size_of_prop(PropertyName::BORDER_BOTTOM);

        let width = match state.size_prop_as_opt(PropertyName::WIDTH) {
            Some(w) => {
                flags = flags | dim::WIDTH_FIXED;
                w
            }
            None => 0f32
        };

        let height = match state.size_prop_as_opt(PropertyName::HEIGHT) {
            Some(h) => {
                flags = flags | dim::HEIGHT_FIXED;
                h
            }
            None => 0f32
        };

        // TODO: Missing bit for left / right / top / bottom
        //       We also need at some point the relative information
        self.dim.content = Rect {
            x: 0f32,
            y: 0f32,
            width: width,
            height: height,
        };
        self.dim.padding = EdgeSizes {
            left: padding_left,
            right: padding_right,
            top: padding_top,
            bottom: padding_bottom,
        };
        self.dim.border = EdgeSizes {
            left: border_left,
            right: border_right,
            top: border_top,
            bottom: border_bottom
        };
        self.dim.margin = EdgeSizes {
            left: margin_left,
            right: margin_right,
            top: margin_top,
            bottom: margin_bottom
        };
        self.flags = flags;
    }
}
