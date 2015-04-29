use std::collections::HashMap;

use util::BufferFromTree;
use uil_shared::style::Value;
use uil_shared::style::SelectorState;
use uil_shared::properties::PropertyName;
use style::StyledNode;

pub struct StateBuffer {
    state_data: BufferFromTree<StateData>,
}

impl StateBuffer {

    pub fn new(style_tree: &StyledNode) -> StateBuffer {

        let size = style_tree.tree_size();

        StateBuffer {
            state_data: BufferFromTree::new(style_tree, size, converter)
        }
    }
}

fn converter(node: &StyledNode) -> Option<StateData> {
    Some(StateData::new(node))
}


pub struct StateData {
    default_properties: HashMap<PropertyName, Value>,
    focus_properties: HashMap<PropertyName, Value>,
    hover_properties: HashMap<PropertyName, Value>,
    creation_properties: HashMap<PropertyName, Value>,
}


impl StateData {

    fn new(node: &StyledNode) -> StateData {

        StateData {
            default_properties: HashMap::new(),
            focus_properties: HashMap::new(),
            hover_properties: HashMap::new(),
            creation_properties: HashMap::new(),
        }
    }
}
