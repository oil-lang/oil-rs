
use std::io::BufRead;

use report::ErrorReporter;
use oil_shared::deps::StyleDefinitions;
use oil_shared::resource::BasicResourceManager;
use oil_shared::style::Stylesheet;


mod parser;

/// Convenient function to parse a style.
pub fn parse<'a, 'b, R, E, B>(
    reporter: E,
    reader: B,
    defs: &'a StyleDefinitions,
    resource_manager: &'b mut R) -> Stylesheet
    where E: ErrorReporter,
          B: BufRead,
          R: BasicResourceManager
{
    let mut parser = parser::Parser::new(reporter, reader, defs, resource_manager);
    parser.parse()
}
