/// Dimensions for the box model.
///
/// This code follows the css box model
/// in naming and conventions. (all sizes are in pixels)
#[derive(Copy, Clone)]
pub struct Dimensions {
    // Position of the content area relative to the viewport origin
    pub content: Rect,
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes,
}

#[derive(Copy, Clone, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Copy, Clone)]
pub struct EdgeSizes {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

bitflags! {
    flags DimFlags: u32 {
        // A text node is WIDTH_FIXED,
        // A node with a style fixed width is naturally WIDTH_FIXED
        const ABSOLUTE_POSITIONING  = 0b10000000,
        const HEIGHT_FIXED          = 0b01000000,
        const WIDTH_FIXED           = 0b00100000,
        const MARGIN_BOT_AUTO       = 0b00010000,
        const MARGIN_TOP_AUTO       = 0b00001000,
        const MARGIN_RIGHT_AUTO     = 0b00000100,
        const MARGIN_LEFT_AUTO      = 0b00000010,
        const WIDTH_AUTO            = 0b00000001,
        const MARGIN_X_AUTO         = MARGIN_LEFT_AUTO.bits
                                    | MARGIN_RIGHT_AUTO.bits,
        const MARGIN_Y_AUTO         = MARGIN_TOP_AUTO.bits
                                    | MARGIN_BOT_AUTO.bits,
    }
}

impl DimFlags {

    #[inline]
    pub fn is_auto(&self) -> bool {
        self.intersects(WIDTH_AUTO | MARGIN_X_AUTO)
    }

    #[inline]
    pub fn has_width_auto(&self) -> bool {
        self.contains(WIDTH_AUTO)
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
    pub fn has_margin_left_auto(&self) -> bool {
        self.contains(MARGIN_LEFT_AUTO)
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
