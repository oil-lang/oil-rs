#![allow(dead_code)]
use super::Rect;

/// Dimensions for the box model.
///
/// This code follows the css box model
/// in naming and conventions. (all sizes are in pixels)
#[derive(Copy, Clone, Default)]
pub struct Dimensions {
    // Position of the content area relative to the viewport origin
    pub content: Rect,
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes,
}

#[derive(Copy, Clone, Default)]
pub struct EdgeSizes {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

bitflags! {
    #[derive(Default)]
    flags DimFlags: u16 {
        // A text node is WIDTH_FIXED,
        // A node with a style fixed width is naturally WIDTH_FIXED
        const ABSOLUTE_POSITIONING  = 0b0100_0000_0000,
        const HEIGHT_FIXED          = 0b0010_0000_0000,
        const WIDTH_FIXED           = 0b0001_0000_0000,

        const MARGIN_RIGHT_EXPAND   = 0b0000_1000_0000,
        const MARGIN_LEFT_EXPAND    = 0b0000_0100_0000,
        const WIDTH_EXPAND          = 0b0000_0010_0000,

        const MARGIN_BOT_AUTO       = 0b0000_0001_0000,
        const MARGIN_TOP_AUTO       = 0b0000_0000_1000,
        const MARGIN_RIGHT_AUTO     = 0b0000_0000_0100,
        const MARGIN_LEFT_AUTO      = 0b0000_0000_0010,
        const WIDTH_AUTO            = 0b0000_0000_0001,

        const MARGIN_X_AUTO         = MARGIN_LEFT_AUTO.bits
                                    | MARGIN_RIGHT_AUTO.bits,
        const MARGIN_Y_AUTO         = MARGIN_TOP_AUTO.bits
                                    | MARGIN_BOT_AUTO.bits,
        const MARGIN_X_EXPAND       = MARGIN_LEFT_EXPAND.bits
                                    | MARGIN_RIGHT_EXPAND.bits,
    }
}

impl DimFlags {

    #[inline]
    pub fn is_x_auto(&self) -> bool {
        self.intersects(WIDTH_AUTO | MARGIN_X_AUTO)
    }

    #[inline]
    pub fn is_x_expand(&self) -> bool {
        self.intersects(WIDTH_EXPAND | MARGIN_X_EXPAND)
    }

    pub fn is_new_line_forced(&self) -> bool {
        self.is_x_auto() || self.is_x_expand()
    }

    #[inline]
    pub fn has_width_auto(&self) -> bool {
        self.contains(WIDTH_AUTO)
    }

    #[inline]
    pub fn has_width_expand(&self) -> bool {
        self.contains(WIDTH_EXPAND)
    }

    #[inline]
    pub fn has_width_fixed(&self) -> bool {
        self.contains(WIDTH_FIXED)
    }

    #[inline]
    pub fn has_height_fixed(&self) -> bool {
        self.contains(HEIGHT_FIXED)
    }

    #[inline]
    pub fn has_margin_top_or_bot_auto(&self) -> bool {
        self.intersects(MARGIN_Y_AUTO)
    }

    #[inline]
    pub fn has_margin_left_expand(&self) -> bool {
        self.intersects(MARGIN_LEFT_EXPAND)
    }

    #[inline]
    pub fn has_margin_left_auto(&self) -> bool {
        self.contains(MARGIN_LEFT_AUTO)
    }

    #[inline]
    pub fn has_margin_right_expand(&self) -> bool {
        self.intersects(MARGIN_RIGHT_EXPAND)
    }

    #[inline]
    pub fn has_margin_right_auto(&self) -> bool {
        self.contains(MARGIN_RIGHT_AUTO)
    }

    #[inline]
    pub fn has_margin_top_auto(&self) -> bool {
        self.contains(MARGIN_TOP_AUTO)
    }

    #[inline]
    pub fn has_margin_bottom_auto(&self) -> bool {
        self.contains(MARGIN_BOT_AUTO)
    }
}
