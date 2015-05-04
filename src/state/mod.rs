use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use util::BufferFromTree;
use uil_shared::asset::ImageData;
use uil_shared::properties::PropertyName;
use uil_shared::properties::STYLE_PROPERTIES;
use uil_shared::style::Value;
use uil_shared::style::KwValue;
use uil_shared::style::Stylesheet;
use uil_shared::style::SelectorState;
use uil_shared::style::Unit;
use uil_shared::style::Rule;
use uil_shared::markup::Node;
use rendering::TextureRule;


pub struct StateBuffer {
    state_data: BufferFromTree<StateData>,
}

impl Deref for StateBuffer {
    type Target = BufferFromTree<StateData>;

    fn deref<'a>(&'a self) -> &'a BufferFromTree<StateData> {
        &self.state_data
    }
}

impl DerefMut for StateBuffer {

    fn deref_mut<'a>(&'a mut self) -> &'a mut BufferFromTree<StateData> {
        &mut self.state_data
    }
}

impl StateBuffer {

    pub fn new(tree: &Node, style_sheet: &Stylesheet) -> StateBuffer {

        let size = tree.tree_size();

        let converter = |node: &Node| {
            Some(StateData::new(node, style_sheet))
        };

        StateBuffer {
            state_data: BufferFromTree::new(tree, size, converter)
        }
    }
}

pub struct StateData {
    default_properties: HashMap<PropertyName, Value>,
    focus_properties: HashMap<PropertyName, Value>,
    hover_properties: HashMap<PropertyName, Value>,
    creation_properties: HashMap<PropertyName, Value>,
    current_state: SelectorState,
}

// ======================================== //
//                  HELPERS                 //
// ======================================== //

// Thanks to lastest rustc nightly macros can't be
// defined at the end of the file anymore. Shame.
macro_rules! return_length_or_zero {

    ($this:ident try $prop_name:ident) => {
        return_length_or_zero!(rec $this try $prop_name else {
            return_length_or_zero!(rec_default $this try $prop_name else { 0f32 })
        });
    };

    ($this:ident try $prop_name:ident else $other:ident) => {
        return_length_or_zero!(rec $this try $prop_name else {
            return_length_or_zero!(rec $this try $other else {
                return_length_or_zero!(rec_default $this try $prop_name else {
                    return_length_or_zero!(rec_default $this try $other else { 0f32 })
                })
            })
        });
    };

    (rec $this:ident try $prop_name:ident else $none_case:block) => {
        match $this.current_properties().get(&$prop_name) {
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

    (rec_default $this:ident try $prop_name:ident else $none_case:block) => {
        match $this.default_properties.get(&$prop_name) {
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

impl StateData {

    fn new(node: &Node, style: &Stylesheet) -> StateData {

        let mut state = StateData {
            default_properties: HashMap::new(),
            focus_properties: HashMap::new(),
            hover_properties: HashMap::new(),
            creation_properties: HashMap::new(),
            current_state: SelectorState::Default
        };

        state.set_properties(node, style);

        state
    }

    pub fn set_current_state(&mut self, new_state: SelectorState) {
        self.current_state = new_state;
    }

    pub fn has_property_expand(&self, prop_name: PropertyName) -> bool {
        self.has_property_eq_kw(prop_name, KwValue::Expand)
    }

    pub fn has_property_auto(&self, prop_name: PropertyName) -> bool {
        self.has_property_eq_kw(prop_name, KwValue::Auto)
    }

    pub fn has_property_eq_kw(&self, prop_name: PropertyName, kw: KwValue) -> bool {
        match self.current_properties().get(&prop_name) {
            Some(&Value::Keyword(v)) if v == kw => {
                true
            },
            _ => match self.default_properties.get(&prop_name) {
                Some(&Value::Keyword(v)) if v == kw => {
                    true
                },
                _ => false
            }
        }
    }

    pub fn get_background_rule(&self) -> Option<TextureRule> {
        match self.current_properties().get(&PropertyName::BACKGROUND_IMAGE_RULE) {
            Some(&Value::Keyword(v)) => match v {
                KwValue::Fit => Some(TextureRule::Fit),
                KwValue::Repeat => Some(TextureRule::Repeat),
                _ => None
            },
            _ => match self.default_properties.get(&PropertyName::BACKGROUND_IMAGE_RULE) {
                Some(&Value::Keyword(v)) => match v {
                    KwValue::Fit => Some(TextureRule::Fit),
                    KwValue::Repeat => Some(TextureRule::Repeat),
                    _ => None
                },
                _ => None
            }
        }
    }

    pub fn get_background_image(&self) -> Option<ImageData> {
        match self.current_properties().get(&PropertyName::BACKGROUND_IMAGE) {
            Some(&Value::Image(ref id)) => Some(id.clone()),
            _ => match self.default_properties.get(&PropertyName::BACKGROUND_IMAGE) {
                Some(&Value::Image(ref id)) => Some(id.clone()),
                _ => None
            }
        }
    }

    pub fn size_prop_as_opt(&self, prop_name: PropertyName) -> Option<f32> {
        match self.current_properties().get(&prop_name) {
            Some(v) => {
                if let Value::Length(val, Unit::Px) = *v {
                    Some(val)
                } else {
                    None
                }
            }
            None => match self.default_properties.get(&prop_name) {
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
    }

    pub fn size_of_prop(&self, prop_name: PropertyName) -> f32 {
        use uil_shared::properties::PropertyName::MARGIN;
        use uil_shared::properties::PropertyName::PADDING;
        use uil_shared::properties::PropertyName::BORDER;

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

    fn current_properties<'a>(&'a self) -> &HashMap<PropertyName, Value> {
        match self.current_state {
            SelectorState::Focus => &self.focus_properties,
            SelectorState::Hover => &self.hover_properties,
            SelectorState::Creation => &self.creation_properties,
            SelectorState::Default => &self.default_properties,
        }
    }

    fn set_properties(&mut self, node: &Node, style: &Stylesheet) {
        let classes = node.classes();
        // We loop over rules because at some
        // point, we might want to sort them based
        // on specificity in the same way that it is done
        // in CSS. It would help understanding which
        // rule does define a particular property.
        // Thus the code below wouldn't change.
        for rule in style.rules.iter() {
            // FIXME:
            // using deref instead of as_ref because
            // of an inference problem.
            if classes.contains(rule.selector.name.deref()) {

                let ref mut properties = match rule.selector.state {
                    SelectorState::Focus => &mut self.focus_properties,
                    SelectorState::Hover => &mut self.hover_properties,
                    SelectorState::Creation => &mut self.creation_properties,
                    SelectorState::Default => &mut self.default_properties
                };
                // Loop over declaration in the rule.
                // If some properties are declared multiple times
                // the order matters here.
                StateData::set_properties_for_hashmap(
                    properties,
                    rule
                );
            }
        }
    }

    fn set_properties_for_hashmap(
        properties: &mut HashMap<PropertyName, Value>,
        rule: &Rule)
    {
        for dec in rule.declarations.iter() {

            if let Some(property) = STYLE_PROPERTIES.get(dec.name.deref()) {

                properties.insert(*property, dec.value.clone());
            }
        }
    }
}
