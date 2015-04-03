// Dependencies
use xml::attribute::OwnedAttribute;
use std::collections::HashSet;

use super::ErrorType;
use super::ErrorStatus;
use super::lookup_name;
use super::HasNodeChildren;

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

impl HasNodeChildren for Node {
    fn add(&mut self, maybe_child: Option<Node>) {
        match maybe_child {
            Some(child) => self.children.push(child),
            None => ()
        }
    }
}

// To help readability:
pub type ResOrError = Result<NodeType, super::ParseError>;

// ------------------------------------------------- Button tag
#[derive(PartialEq, Clone, Debug)]
pub struct ButtonData {
    pub gotoview: Option<String>,
    pub action: Option<String>,
    pub key: Option<String>,
}

pub fn parse_button(attributes: &Vec<OwnedAttribute>) -> ResOrError {
    Ok(NodeType::Button(ButtonData {
        gotoview: lookup_name("goto-view", attributes),
        action: lookup_name("action", attributes),
        key: lookup_name("key", attributes),
    }))
}

// ------------------------------------------------- Line input tag
#[derive(PartialEq, Clone, Debug)]
pub struct LineInputData {
    pub value: Option<String>,
    pub key: Option<String>,
}

pub fn parse_linput(attributes: &Vec<OwnedAttribute>) -> ResOrError {
    Ok(NodeType::LineInput(LineInputData {
        value: lookup_name("value", attributes),
        key: lookup_name("key", attributes),
    }))
}

// ------------------------------------------------- Progress bar tag
#[derive(PartialEq, Clone, Debug)]
pub struct ProgressBarData {
    pub value: Option<String>
}

pub fn parse_pbar(attributes: &Vec<OwnedAttribute>) -> ResOrError {
    Ok(NodeType::ProgressBar(ProgressBarData {
        value: lookup_name("value", attributes)
    }))
}

// ------------------------------------------------- Template tag
#[derive(PartialEq, Clone, Debug)]
pub struct TemplateData {
    pub path: String,
}

pub fn parse_template(attributes: &Vec<OwnedAttribute>) -> ResOrError {
    match lookup_name("path", attributes) {
        Some(path) => {
            Ok(NodeType::Template(TemplateData {
                path: path
            }))
        }
        None => {
            Err((
                ErrorType::Warning,
                ErrorStatus::NotReported(
                    "`path` attribute in `template` is missing")
            ))
        }
    }
}

// ------------------------------------------------- Repeat tag
#[derive(PartialEq, Clone, Debug)]
pub struct RepeatData {
    pub template_name: String,
    pub iter: String,
}

pub fn parse_repeat(attributes: &Vec<OwnedAttribute>) -> ResOrError {

    match (lookup_name("template-name", attributes),
           lookup_name("iter", attributes))
    {
        (Some(name), Some(iter)) => {
            Ok(NodeType::Repeat(RepeatData {
                template_name: name,
                iter: iter
            }))
        }
        (None, _) => {
            Err((
                ErrorType::Warning,
                ErrorStatus::NotReported(
                    "`template-name` attribute in `repeat` is missing")
            ))
        }
        (_, None) => {
            Err((
                ErrorType::Warning,
                ErrorStatus::NotReported(
                    "`iter` attribute in `repeat` is missing")
            ))
        }
    }
}
