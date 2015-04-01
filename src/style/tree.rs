use std::collections::HashMap;
use markup::Node;
use rendering::TextureRule;
use asset::ImageData;
use super::Value;
use super::Stylesheet;
use super::Unit;
use phf;

/// List of style properties
///
/// If you do a change here, you must update STYLE_PROPERTIES
#[derive(Copy, PartialEq, Eq, Hash)]
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

pub struct StyledNode<'a> {
    node: &'a Node,
    property_values: HashMap<PropertyName, Value>,
    pub kids: Vec<StyledNode<'a>>,
}

// ======================================== //
//                  HELPERS                 //
// ======================================== //

// Thanks to lastest rustc nightly macros can't be
// defined at the end of the file anymore. Shame.
macro_rules! return_length_or_zero {

    ($this:ident try $prop_name:ident) => {
        return_length_or_zero!(rec $this try $prop_name else { 0f32 });
    };

    ($this:ident try $prop_name:ident else $other:ident) => {
        return_length_or_zero!(rec $this try $prop_name else {
            return_length_or_zero!(rec $this try $other else { 0f32 })
        });
    };

    (rec $this:ident try $prop_name:ident else $none_case:block) => {
        match $this.property_values.get(&$prop_name) {
            Some(v) => {
                if let Value::Length(val, Unit::Px) = *v {
                    val
                } else {
                    0f32
                }
            }
            None => $none_case
        }
    };
}

// ======================================== //
//                 INTERFACE                //
// ======================================== //

pub fn build_style_tree<'a, 'b>(node: &'a Node, stylesheet: &'b Stylesheet) -> StyledNode<'a> {
    let mut styled_node = StyledNode::<'a>::new(node);
    styled_node.set_properties(stylesheet);
    styled_node
}


impl<'a> StyledNode<'a> {

    fn new(node: &'a Node) -> StyledNode<'a> {
        let mut kids = Vec::with_capacity(node.children.len());
        for kid in node.children.iter() {
            kids.push(StyledNode::new(kid));
        }

        StyledNode {
            node: node,
            property_values: HashMap::new(),
            kids: kids
        }
    }

    pub fn is_property_auto(&self, prop_name: PropertyName) -> bool {
        match self.property_values.get(&prop_name) {
            Some(&Value::KeywordAuto) => {
                true
            },
            _ => false
        }
    }

    pub fn get_background_rule(&self) -> Option<TextureRule> {
        match self.property_values.get(&PropertyName::BACKGROUND_IMAGE_RULE) {
            Some(&Value::KeywordFit) => Some(TextureRule::Fit),
            Some(&Value::KeywordRepeat) => Some(TextureRule::Repeat),
            _ => None
        }
    }

    pub fn get_background_image(&self) -> Option<ImageData> {
        match self.property_values.get(&PropertyName::BACKGROUND_IMAGE) {
            Some(&Value::Image(ref id)) => Some(id.clone()),
            _ => None
        }
    }

    pub fn size_prop_as_opt(&self, prop_name: PropertyName) -> Option<f32> {
        match self.property_values.get(&prop_name) {
            Some(v) => {
                if let Value::Length(val, Unit::Px) = *v {
                    Some(val)
                } else {
                    None
                }
            }
            None => None
        }
    }

    pub fn size_prop(&self, prop_name: PropertyName) -> f32 {
        use self::PropertyName::MARGIN;
        use self::PropertyName::PADDING;
        use self::PropertyName::BORDER;

        match prop_name {
            PropertyName::LEFT
            | PropertyName::RIGHT
            | PropertyName::TOP
            | PropertyName::BOTTOM
            | PropertyName::HEIGHT
            | PropertyName::WIDTH => {
                return_length_or_zero!(self try prop_name)
            }
            PropertyName::MARGIN_LEFT
            | PropertyName::MARGIN_RIGHT
            | PropertyName::MARGIN_TOP
            | PropertyName::MARGIN_BOTTOM => {
                return_length_or_zero!(self try prop_name else MARGIN)
            }
            PropertyName::PADDING_LEFT
            | PropertyName::PADDING_RIGHT
            | PropertyName::PADDING_TOP
            | PropertyName::PADDING_BOTTOM => {
                return_length_or_zero!(self try prop_name else PADDING)
            }
            PropertyName::BORDER_LEFT
            | PropertyName::BORDER_RIGHT
            | PropertyName::BORDER_TOP
            | PropertyName::BORDER_BOTTOM => {
                return_length_or_zero!(self try prop_name else BORDER)
            }
            _ => panic!()
        }
    }

    pub fn tree_size(&self) -> usize {
        let mut count = 1;

        for kid in &self.kids {
            count += kid.tree_size();
        }

        count
    }

    fn set_properties(&mut self, style: &Stylesheet) {
        let classes = self.node.classes();
        let ref mut properties = self.property_values;
        // We loop over rules because at some
        // point, we might want to sort them based
        // on specificity in the same way that it is done
        // in CSS. It would help understanding which
        // rule does define a particular property.
        // Thus the code below wouldn't change.
        for rule in style.rules.iter() {
            if classes.contains(rule.selector.as_slice()) {
                // Loop over declaration in the rule.
                // If some properties are declared multiple times
                // the order matters here.
                for dec in rule.declarations.iter() {

                    if let Some(property) = STYLE_PROPERTIES.get(dec.name.as_slice()) {

                        properties.insert(*property, dec.value.clone());
                    }
                }
            }
        }

        for kid in self.kids.iter_mut() {
            kid.set_properties(style);
        }
    }
}

static STYLE_PROPERTIES: phf::Map<&'static str, PropertyName> = phf_map! {
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
