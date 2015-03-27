
mod parser;

// Dependencies
use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;
use report::ErrorReporter;
use style;
use asset;

/// Convenient function to parse a style.
pub fn parse<E, B>(reporter: E, reader: B) -> StyleDefinitions
    where E: ErrorReporter,
          B: BufRead
{
    let mut parser = parser::Parser::new(reporter, reader);
    parser.parse()
}

pub struct StyleDefinitions {
    pub defs: HashMap<String, Constructor>,
}

impl StyleDefinitions {
    pub fn new() -> StyleDefinitions {
        StyleDefinitions {
            defs: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Constructor {
    /// None type -> Constructor failed loading the resource.
    None,
    /// Number [0-9]+
    Number(f32),
    /// String ".+"
    Quote(String),
    /// Font(path, width, height)
    Font(String, f32, f32),
    /// TODO: replace String by the type Path
    /// Image(path, width, height, offset-x, offset-y)
    Image(PathBuf, Option<f32>, Option<f32>, Option<f32>, Option<f32>),
    // Add other construtor here...
}

impl Constructor {
    pub fn convert_to_style_value(&self) -> Option<style::Value> {
        // TODO: FIXME
        // A string should be converted into Keyword(String),
        // once the modification is done to style::Value.
        //
        match *self {
            Constructor::Number(v) => Some(style::Value::Length(v, style::Unit::Px)),
            Constructor::Quote(..) => Some(style::Value::KeywordAuto),
            Constructor::Font(..) => Some(style::Value::Font(asset::FontData::new(self))),
            Constructor::Image(..) => Some(style::Value::Image(asset::ImageData::new(self))),
            Constructor::None => None,
        }
    }
}
