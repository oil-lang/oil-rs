use std::collections::HashSet;

#[derive(PartialEq, Clone, Debug)]
pub enum NodeType {
    Text(String),
    Group,
    Button(ButtonData),
    LineInput(LineInputData),
    ProgressBar(ProgressBarData),
    Template(TemplateData),
    Repeat(RepeatData),
    // Special Root Nodes
    RootView,
    RootTemplate
}

#[derive(Clone, Debug)]
pub struct Node {
    pub children: Vec<Node>,
    classes: Option<String>,
    pub node_type: NodeType,
}

pub type Template = Node;
pub type View = Node;

pub fn new_template(classes: Option<String>) -> Template {
    Node {
        children: Vec::new(),
        node_type: NodeType::RootTemplate,
        classes: classes
    }
}

pub fn new_view(classes: Option<String>) -> View {
    Node {
        children: Vec::new(),
        node_type: NodeType::RootView,
        classes: classes,
    }
}

impl Node {

    pub fn new(classes: Option<String>, nt: NodeType) -> Node {
        Node {
            children: Vec::new(),
            node_type: nt,
            classes: classes,
        }
    }

    pub fn from_template(other: &Template, nt: NodeType) -> Node {
        Node {
            children: other.children.clone(),
            node_type: nt,
            classes: None
        }
    }

    pub fn classes(&self) -> HashSet<&str> {
        match self.classes {
            Some(ref classlist) => classlist.split(' ').collect(),
            None => HashSet::new()
        }
    }
}

// ------------------------------------------------- Button tag
#[derive(PartialEq, Clone, Debug)]
pub struct ButtonData {
    pub gotoview: Option<String>,
    pub action: Option<String>,
    pub key: Option<String>,
}

// ------------------------------------------------- Line input tag
#[derive(PartialEq, Clone, Debug)]
pub struct LineInputData {
    pub value: Option<String>,
    pub key: Option<String>,
}

// ------------------------------------------------- Progress bar tag
#[derive(PartialEq, Clone, Debug)]
pub struct ProgressBarData {
    pub value: Option<String>
}

// ------------------------------------------------- Template tag
#[derive(PartialEq, Clone, Debug)]
pub struct TemplateData {
    pub path: String,
}

// ------------------------------------------------- Repeat tag
#[derive(PartialEq, Clone, Debug)]
pub struct RepeatData {
    pub template_name: String,
    pub iter: String,
}
