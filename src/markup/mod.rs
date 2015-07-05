

// Re-export
pub use oil_shared::markup::Node;
pub use oil_shared::markup::NodeType;
pub use oil_shared::markup::{Template, View};
pub use oil_shared::markup::{
    ButtonData,
    LineInputData,
    ProgressBarData,
    TemplateData,
    RepeatData
};

pub use oil_parsers::markup::Library;
pub use oil_parsers::markup::MAIN_VIEW_NAME;
pub use oil_parsers::markup::parse;

use util::HasChildren;

impl HasChildren for Node {

    fn children<'b>(&'b self) -> &'b [Node] {
        &self.children
    }
}
