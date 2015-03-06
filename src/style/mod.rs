
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


/// Convenient function to parse a style.
pub fn parse<'a, E, B>(reporter: E, reader: B, defs: &'a StyleDefinitions) -> Stylesheet
    where E: ErrorReporter,
          B: Buffer
{
    let mut parser = parser::Parser::new(reporter, reader, defs);
    parser.parse()
}
