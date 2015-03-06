
mod parser;

// Dependencies
use std::collections::HashMap;
use report::ErrorReporter;
use style;
use asset;

/// Convenient function to parse a style.
pub fn parse<E, B>(reporter: E, reader: B) -> StyleDefinitions
    where E: ErrorReporter,
          B: Buffer
{
    let mut parser = parser::Parser::new(reporter, reader);
    parser.parse()
}

pub struct StyleDefinitions {
    pub defs: HashMap<String, Value>,
}

impl StyleDefinitions {
    pub fn new() -> StyleDefinitions {
        StyleDefinitions {
            defs: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    /// None type -> Constructor failed loading the resource.
    None,
    /// Number [0-9]+
    Number(f32),
    /// String ".+"
    Quote(String),
    /// Font(path, width, height)
    Font(String, f32, f32),
    /// Image(path)
    Image(String),
    // Add other construtor here...
}

impl Value {
    pub fn convert_to_style_value(&self) -> Option<style::Value> {
        // TODO: FIXME
        // A string should be converted into Keyword(String),
        // once the modification is done to style::Value.
        //
        match *self {
            Value::Number(v) => Some(style::Value::Length(v, style::Unit::Px)),
            Value::Quote(..) => Some(style::Value::KeywordAuto),
            Value::Font(..) => Some(style::Value::Font(asset::FontData)),
            Value::Image(..) => Some(style::Value::Image(asset::ImageData)),
            Value::None => None,
        }
    }
}
