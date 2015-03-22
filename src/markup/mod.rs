
// Re-export
pub use self::tags::Node;
pub use self::tags::NodeType;
pub use self::tags::{Template, View};
pub use self::tags::{
    ButtonData,
    LineInputData,
    ProgressBarData,
    TemplateData,
    RepeatData
};
pub use self::lib::Library;

mod lib;
mod tags;
mod parser;

// Name for the "main" view.
pub const MAIN_VIEW_NAME: &'static str = "main";

// Tag list
const TEMPLATE_TAG: &'static str = "template";
const VIEW_TAG: &'static str = "view";
const GROUP_TAG: &'static str = "group";
const BUTTON_TAG: &'static str = "button";
const LINE_INPUT_TAG: &'static str = "line-input";
const PROGRESS_BAR_TAG: &'static str = "progress-bar";
const REPEAT_TAG: &'static str = "repeat";

// Dependencies
use self::parser::Parser;
use xml::attribute::OwnedAttribute;
use ErrorReporter;


/// Parse the given buffer.
///
/// # Example:
///
/// ```
/// use uil::StdOutErrorReporter;
/// use uil::markup;
///
/// let reader = std::old_io::BufferedReader::new(
///     "<view name=\"toto\">\
///     </view>\
/// ".as_bytes());
/// markup::parse(StdOutErrorReporter, reader);
/// ```
pub fn parse<E, B>(reporter: E, reader: B) -> Library<E>
    where E: ErrorReporter,
          B: Buffer
{
    let mut parser = Parser::new(reporter, reader);
    parser.parse()
}

pub trait HasNodeChildren {
    fn add(&mut self, maybe_child: Option<Node>);
}

// ======================================== //
//                  HELPERS                 //
// ======================================== //

fn lookup_name<'a>(name: &'a str,
                   attributes: &Vec<OwnedAttribute>)
                   -> Option<String>
{
    attributes.iter()
        .find(|ref attribute| attribute.name.local_name == name)
        .map(|ref attribute| attribute.value.clone())
}

enum ErrorStatus {
    NotReported(&'static str),
    Reported,
}

enum ErrorType {
    Fatal,
    Warning,
}

type ParseError = (ErrorType, ErrorStatus);
