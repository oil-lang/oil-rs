
use std::collections::HashMap;

use oil_shared::markup::{
    Node, NodeType, TemplateData, View, Template
};
use ErrorReporter;

// Library
pub struct Library<E> {
    pub views: HashMap<String, View>,
    pub templates: HashMap<String, Template>,
    err: E,
}

impl<E> Library<E>
    where E: ErrorReporter
{

    pub fn new(reporter: E,
               views: HashMap<String, View>,
               templates: HashMap<String, Template>) -> Library<E>
    {
        Library {
            err: reporter,
            views: views,
            templates: templates
        }
    }

    pub fn get<S: ToString>(&self, s: S) -> Option<&View> {
        self.views.get(&s.to_string())
    }

    pub fn merge(&mut self, other: Library<E>) {
        for (key, val) in other.views.into_iter() {
            self.views.insert(key, val);
        }

        for (key, val) in other.templates.into_iter() {
            self.templates.insert(key, val);
        }
    }

    /// # Resolve the dependencies
    ///
    /// Convert all templates that match a template definition
    /// with a group containing the templates childs.
    ///
    /// Note: this does not resolve data-bindings dependencies.
    //#[deprecated(reason = "This is now managed automatically when the view is created.",
    //         	 since = "0.2.0")]
    pub fn resolve_templates(&mut self) {
        let ref mut views = self.views;
        let ref templates = self.templates;
        let ref err = self.err;
        for (_, view) in views.iter_mut() {
            for node in view.children.iter_mut() {
                Library::<E>::resolve_templates_for_node(
                    &err,
                    &templates,
                    node
                );
            }
        }
    }

    fn resolve_templates_for_node(err: &E,
                                  templates: &HashMap<String, Template>,
                                  node: &mut Node)
    {
        let new_node_opt = match node.node_type {
            NodeType::Template(TemplateData { ref path }) => {
                match templates.get(path) {
                    None => {
                        err.log(format!(
                                "Warning `{}` template name not found", path));
                        None
                    }
                    Some(found) => Some(Node::from_template(found, NodeType::Group)),
                }
            }
            _ => None
        };

        if let Some(new_node) = new_node_opt {
            *node = new_node;
        }

        for child in node.children.iter_mut() {
            Library::<E>::resolve_templates_for_node(err, templates, child);
        }
    }
}
