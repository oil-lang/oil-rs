
pub use self::rules::Stylesheet;
pub use self::rules::Rule;
pub use self::rules::Declaration;
pub use self::rules::Value;
pub use self::rules::Unit;
pub use self::tree::StyledNode;
pub use self::tree::PropertyName;
pub use self::tree::build_style_tree;

mod rules;
mod parser;
mod tree;

use report::ErrorReporter;
use deps::StyleDefinitions;
use resource::ResourceManager;
use std::io::BufRead;

/// Convenient function to parse a style.
pub fn parse<'a, 'b, R, E, B>(
    reporter: E,
    reader: B,
    defs: &'a StyleDefinitions,
    resource_manager: &'b mut R) -> Stylesheet
    where E: ErrorReporter,
          B: BufRead,
          R: ResourceManager
{
    let mut parser = parser::Parser::new(reporter, reader, defs, resource_manager);
    parser.parse()
}
