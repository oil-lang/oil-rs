// Dependencies
use xml::attribute::OwnedAttribute;

use super::ErrorType;
use super::ErrorStatus;
use super::lookup_name;
use super::HasNodeChildren;

use oil_shared::markup::{
    Node,
    NodeType,
    ButtonData,
    LineInputData,
    ProgressBarData,
    TemplateData,
    RepeatData
};


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

pub fn parse_button(attributes: &Vec<OwnedAttribute>) -> ResOrError {
    Ok(NodeType::Button(ButtonData {
        gotoview: lookup_name("goto-view", attributes),
        action: lookup_name("action", attributes),
        key: lookup_name("key", attributes),
    }))
}

// ------------------------------------------------- Line input tag

pub fn parse_linput(attributes: &Vec<OwnedAttribute>) -> ResOrError {
    Ok(NodeType::LineInput(LineInputData {
        value: lookup_name("value", attributes),
        key: lookup_name("key", attributes),
    }))
}

// ------------------------------------------------- Progress bar tag

pub fn parse_pbar(attributes: &Vec<OwnedAttribute>) -> ResOrError {
    Ok(NodeType::ProgressBar(ProgressBarData {
        value: lookup_name("value", attributes)
    }))
}

// ------------------------------------------------- Template tag

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
