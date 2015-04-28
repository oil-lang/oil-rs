use phf;

/// List of style properties
///
/// If you do a change here, you must update STYLE_PROPERTIES
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum PropertyName {
    // Absolute positioning properties
    LEFT,
    RIGHT,
    TOP,
    BOTTOM,
    HEIGHT,
    WIDTH,
    // Margin properties
    MARGIN,
    MARGIN_LEFT,
    MARGIN_RIGHT,
    MARGIN_TOP,
    MARGIN_BOTTOM,
    // Padding properties
    PADDING,
    PADDING_LEFT,
    PADDING_RIGHT,
    PADDING_TOP,
    PADDING_BOTTOM,
    // Border properties
    BORDER,
    BORDER_LEFT,
    BORDER_RIGHT,
    BORDER_TOP,
    BORDER_BOTTOM,
    // Layout mode (absolute / rtl / ltr)
    LAYOUT_MODE,

    /// Background
    /// Possibles rules:
    /// * `"fit"` will scale the image with the node content size.
    /// * `"repeat"` won't scale the image but will repeat it
    ///
    /// In any case, the node content bounds will be the final image bounds
    BACKGROUND_IMAGE_RULE,
    /// This property can only have Value::Image.
    BACKGROUND_IMAGE,
}

pub static STYLE_PROPERTIES: phf::Map<&'static str, PropertyName> = phf_map! {
    // Absolute positioning properties
    "left" => PropertyName::LEFT,
    "right" => PropertyName::RIGHT,
    "top" => PropertyName::TOP,
    "bottom" => PropertyName::BOTTOM,
    "height" => PropertyName::HEIGHT,
    "width" => PropertyName::WIDTH,
    // Margin properties
    "margin" => PropertyName::MARGIN,
    "margin-left" => PropertyName::MARGIN_LEFT,
    "margin-right" => PropertyName::MARGIN_RIGHT,
    "margin-top" => PropertyName::MARGIN_TOP,
    "margin-bottom" => PropertyName::MARGIN_BOTTOM,
    // Padding properties
    "padding" => PropertyName::PADDING,
    "padding-left" => PropertyName::PADDING_LEFT,
    "padding-right" => PropertyName::PADDING_RIGHT,
    "padding-top" => PropertyName::PADDING_TOP,
    "padding-bottom" => PropertyName::PADDING_BOTTOM,
    // Border properties
    "border" => PropertyName::BORDER,
    "border-left" => PropertyName::BORDER_LEFT,
    "border-right" => PropertyName::BORDER_RIGHT,
    "border-top" => PropertyName::BORDER_TOP,
    "border-bottom" => PropertyName::BORDER_BOTTOM,
    // Layout mode (absolute / rtl / ltr)
    "layout" => PropertyName::LAYOUT_MODE,
    // Background image
    "background-image" => PropertyName::BACKGROUND_IMAGE,
    "background-image-rule" => PropertyName::BACKGROUND_IMAGE_RULE,
};
