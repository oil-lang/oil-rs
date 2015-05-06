

// Re-export
pub use uil_shared::markup::Node;
pub use uil_shared::markup::NodeType;
pub use uil_shared::markup::{Template, View};
pub use uil_shared::markup::{
    ButtonData,
    LineInputData,
    ProgressBarData,
    TemplateData,
    UnlinkedRepeatData,
    RepeatBindingData,
};

pub use uil_parsers::markup::Library;
pub use uil_parsers::markup::MAIN_VIEW_NAME;
pub use uil_parsers::markup::parse;

use util::HasChildren;

impl HasChildren for Node {

    fn children<'b>(&'b self) -> &'b [Node] {
        &self.children
    }
}
