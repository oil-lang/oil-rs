// Dependencies
use xml::reader::EventReader;
use xml::reader::events::*;
use xml::attribute::OwnedAttribute;
use std::io::Read;

use std::collections::HashMap;
use ErrorReporter;
use oil_shared;
use parsing::BufferConsumer;

use super::HasNodeChildren;
use super::Library;
use super::ErrorType;
use super::ErrorStatus;
use super::tags;
use super::lookup_name;
use super::ParseError;

use super::TEMPLATE_TAG;
use super::VIEW_TAG;
use super::GROUP_TAG;
use super::BUTTON_TAG;
use super::LINE_INPUT_TAG;
use super::PROGRESS_BAR_TAG;
use super::REPEAT_TAG;
use super::MAIN_VIEW_NAME;

use oil_shared::markup::{
    Node,
    View,
    Template,
    NodeType
};

/// Parser
pub struct Parser<E, B: Read> {
    err: E,
    parser: EventReader<B>,
}

impl<E, B> Parser<E, B>
    where E: ErrorReporter,
          B: Read
{

    pub fn new(reporter: E, reader: B) -> Parser<E, B> {
        Parser {
            err: reporter,
            parser: EventReader::new(reader)
        }
    }

    pub fn parse(&mut self) -> Library<E>
    {
        let mut views = HashMap::new();
        let mut templates = HashMap::new();

        'doc: loop {

            match self.parser.next() {
                XmlEvent::StartElement { name, attributes, .. } => {

                    let test_parse = self.parse_root_tag(
                        &mut views,
                        &mut templates,
                        &name.local_name,
                        &attributes
                    );

                    match test_parse {
                        Err((ErrorType::Fatal, _)) => break 'doc,
                        _ => ()
                    }
                }
                XmlEvent::Error(e) => {
                    self.err.log(format!("Error: {}", e));
                    break 'doc;
                }
                XmlEvent::EndDocument => break 'doc,
                XmlEvent::StartDocument { .. } => (),
                _ => unreachable!(),
            }
        }

        Library::new(self.err.clone(), views, templates)
    }

    fn parse_view(&mut self) -> Result<View, ParseError>
    {
        let mut view = oil_shared::markup::new_view(None);

        try!(self.parse_loop(VIEW_TAG, &mut view));
        Ok(view)
    }

    fn parse_template_decl(&mut self) -> Result<Template, ParseError>
    {
        let mut template = oil_shared::markup::new_template(None);

        try!(self.parse_loop(TEMPLATE_TAG, &mut template));
        Ok(template)
    }

    fn parse_root_tag(&mut self,
                      views: &mut HashMap<String, View>,
                      templates: &mut HashMap<String, Template>,
                      name: &str,
                      attributes: &Vec<OwnedAttribute>) -> Result<(), ParseError>
    {
        match name {
            TEMPLATE_TAG => {
                let attr_name = lookup_name("name", attributes);

                match attr_name {
                    None => {
                        // FIXME(Waiting the pull request on netvl/xml-rs to be merged)
                        // let (row, col) = self.parser.get_cursor();
                        let (row, col) = (0, 0);
                        self.err.log(
                            format!(
                                "Warning {}:{} : `template` has no name add an \
                                 attribute 'name=\"<a-name>\"'",
                            row, col)
                        );

                        self.consume_children(name)
                    }
                    Some(template_name) => {
                        let template = try!(self.parse_template_decl());
                        templates.insert(template_name, template);
                        Ok(())
                    }
                }
            }
            VIEW_TAG => {
                let view = try!(self.parse_view());
                let attr_name = lookup_name("name", attributes)
                    .unwrap_or(MAIN_VIEW_NAME.to_string());
                views.insert(attr_name, view);
                Ok(())
            }
            _ => {
                // FIXME(Waiting the pull request on netvl/xml-rs to be merged)
                // let (row, col) = self.parser.get_cursor();
                let (row, col) = (0, 0);
                self.err.log(
                    format!(
                        "Warning {}:{} : `{}` can't be at root level, \
                        you can only have `template` or `view`"
                    , row+1, col+1, name));

                self.consume_children(name)
            }
        }
    }

    fn parse_tag(&mut self,
                 name: &str,
                 attributes: &Vec<OwnedAttribute>)
                 -> Result<Option<Node>, ParseError>
    {
        let ignore_child = name == TEMPLATE_TAG;

        let node_type = match name {
            TEMPLATE_TAG     => tags::parse_template(attributes),
            GROUP_TAG        => Ok(NodeType::Group),
            BUTTON_TAG       => tags::parse_button(attributes),
            LINE_INPUT_TAG   => tags::parse_linput(attributes),
            PROGRESS_BAR_TAG => tags::parse_pbar(attributes),
            REPEAT_TAG       => tags::parse_repeat(attributes),
            _ => {
                // FIXME(Waiting the pull request on netvl/xml-rs to be merged)
                // let (row, col) = self.parser.get_cursor();
                let (row, col) = (0, 0);
                self.err.log(
                    format!("Warning {}:{} : Unknown tag `{}`", row+1, col+1, name)
                );
                Err((ErrorType::Warning, ErrorStatus::Reported))
            }
        };

        match node_type {
            Err(parse_error) => {
                match self.report_error_if_needed(parse_error) {
                    (ErrorType::Warning, _) => {
                        match self.consume_children(name) {
                            Err(parse_err) =>
                                Err(self.report_error_if_needed(parse_err)),
                            Ok(()) =>
                                Ok(None)
                        }
                    },
                    reported_parse_error => Err(reported_parse_error)
                }
            }
            Ok(nt) => {
                let classes = lookup_name("class", attributes);
                let mut node = Node::new(classes, nt);

                if ignore_child {

                    // Consume children
                    try!(self.consume_children(name));
                    Ok(Some(node))

                } else {

                    // Parse children
                    try!(self.parse_loop(name, &mut node));
                    Ok(Some(node))
                }
            }
        }
    }

    fn parse_data_binding(&mut self, text: &str, parent: &mut Node) {
        let mut buf_consumer = BufferConsumer::new(text.as_bytes());
        loop {
            let text = buf_consumer.consume_while(|c| c != '{').unwrap();
            if !text.is_empty() {
                parent.add(Some(Node::new(
                            None,
                            NodeType::Text(text))));
            }
            match buf_consumer.consume_any_char() {
                Some('{') => {
                    match buf_consumer.consume_any_char() {
                        Some('{') => {
                            let mut data_binding = buf_consumer.consume_while(|c| c != '}').unwrap();
                            match buf_consumer.consume_any_char() {
                                Some('}') => {
                                    match buf_consumer.consume_any_char() {
                                        Some('}') => {
                                            // Yahoo! We have a correct data binding
                                            parent.add(Some(Node::new(
                                                        None,
                                                        NodeType::Binding(data_binding))));
                                        }
                                        _ => {
                                            let (row, col) = (0, 0);
                                            self.err.log(format!("Warning {}:{} : Missing '}}' for data binding", row, col));
                                            data_binding.insert(0, '{');
                                            data_binding.insert(0, '{');
                                            data_binding.push('}');
                                            parent.add(Some(Node::new(
                                                        None,
                                                        NodeType::Text(data_binding))));
                                            break;
                                        },
                                    }
                                }
                                _ => {
                                    let (row, col) = (0, 0);
                                    self.err.log(format!("Warning {}:{} : Missing '}}' for data binding", row, col));
                                    data_binding.insert(0, '{');
                                    data_binding.insert(0, '{');
                                    parent.add(Some(Node::new(
                                                None,
                                                NodeType::Text(data_binding))));
                                    break;
                                },
                            }
                        }
                        _ => {
                            let (row, col) = (0, 0);
                            self.err.log(format!("Warning {}:{} : Missing \"{{ .. }}}}\" for data binding", row, col));
                            let text = "{".to_string();
                            parent.add(Some(Node::new(
                                        None,
                                        NodeType::Text(text))));
                            break;
                        },
                    }
                }
                _ => break,
            }
        }
    }

    fn report_error_if_needed(&mut self,
                              parse_error: ParseError) -> ParseError
    {
        // FIXME(Waiting the pull request on netvl/xml-rs to be merged)
        // let (row, col) = self.parser.get_cursor();
        let (row, col) = (0, 0);
        match parse_error {
            (ErrorType::Fatal, ErrorStatus::NotReported(msg)) => {
                self.err.log(
                    format!("Error {}:{} : {}", row+1, col+1, msg)
                );
                (ErrorType::Fatal, ErrorStatus::Reported)
            }
            (ErrorType::Warning, ErrorStatus::NotReported(msg)) => {
                self.err.log(
                    format!("Warning {}:{} : {}", row+1, col+1, msg)
                );
                (ErrorType::Warning, ErrorStatus::Reported)
            }
            _ => parse_error
        }
    }

    // This function may only return Ok(()) or
    // Err((ErrorType::Fatal, ErrorStatus::Reported)).
    fn consume_children(&mut self, tag: &str) -> Result<(), ParseError>
    {
        let mut depth = 1i32;
        loop {
            match self.parser.next() {
                XmlEvent::StartElement { name, .. } => {

                    depth += 1;

                    // FIXME(Waiting the pull request on netvl/xml-rs to be merged)
                    // let (row, col) = self.parser.get_cursor();
                    let (row, col) = (0, 0);
                    self.err.log(
                        format!("Warning {}:{}, `{}` has been ignored",
                                row+1, col+1, name)
                    );
                }
                XmlEvent::EndElement { name } => {

                    depth -= 1;
                    if name.local_name == tag && depth == 0 {
                        return Ok(());
                    }
                }
                XmlEvent::Error( e ) => {

                    self.err.log(format!("Error: {}", e));
                    return Err((ErrorType::Fatal, ErrorStatus::Reported));
                }
                _ => ()
            }
        }
    }

    fn parse_loop(&mut self,
                tag: &str,
                parent: &mut Node)
                -> Result<(), ParseError>
    {
        loop {
            match self.parser.next() {
                XmlEvent::StartElement { name, attributes, .. } => {

                    let test_parse_child = self.parse_tag(
                        &name.local_name,
                        &attributes
                    );

                    match test_parse_child {
                        // We're fine continue parsing.
                        Ok(node) => {
                            parent.add(node);
                        },
                        // Error has been reported: stop parsing.
                        Err(reported_error) => return Err(reported_error),
                    }
                }
                XmlEvent::EndElement { name } => {

                    // TODO: remove at some point.
                    assert_eq!(name.local_name, tag);
                    return Ok(());
                }
                XmlEvent::Characters( text ) => {
                    self.parse_data_binding(&text, parent);
                }
                XmlEvent::Error( e ) => {

                    self.err.log(format!("Error: {}", e));
                    return Err((ErrorType::Fatal, ErrorStatus::Reported));
                }
                XmlEvent::EndDocument => unreachable!(),
                _ => ()
            }
        }
    }

}

#[cfg(test)]
mod test {

    use std::io::BufReader;
    use EmptyErrorReporter;

    #[test]
    fn reject_invalid_root_tags() {
        let reader = BufReader::new("<test></test>".as_bytes());
        let mut parser = super::Parser::new(EmptyErrorReporter, reader);

        let res = parser.parse();
        assert_eq!(res.views.len(), 0);
        assert_eq!(res.templates.len(), 0);
    }

    #[test]
    fn ignore_unknown_tags() {
        let reader = BufReader::new(
            "<view>\
                <toto />\
                <h1>Test</h1>\
             </view>
            ".as_bytes());
        let mut parser = super::Parser::new(EmptyErrorReporter, reader);

        let res = parser.parse();

        assert_eq!(res.views.len(), 1);
        assert_eq!(res.views.values().next().unwrap().children.len(), 0);
        assert_eq!(res.templates.len(), 0);
    }

    #[test]
    fn reject_unnamed_template() {
        let reader = BufReader::new(
            "<template>\
                <toto />\
             </template>
            ".as_bytes());
        let mut parser = super::Parser::new(EmptyErrorReporter, reader);

        let res = parser.parse();

        assert_eq!(res.views.len(), 0);
        assert_eq!(res.templates.len(), 0);
    }

    #[test]
    fn ignore_ill_formed_repeat_1() {
        let reader = BufReader::new(
            "<view>\
                <repeat template-name=\"test\"/>\
             </view>
            ".as_bytes());
        let mut parser = super::Parser::new(EmptyErrorReporter, reader);

        let res = parser.parse();

        assert_eq!(res.views.len(), 1);
        assert_eq!(res.views.values().next().unwrap().children.len(), 0);
        assert_eq!(res.templates.len(), 0);
    }

    #[test]
    fn ignore_ill_formed_repeat_2() {
        let reader = BufReader::new(
            "<view>\
                <repeat iter=\"{test}\"/>\
             </view>
            ".as_bytes());
        let mut parser = super::Parser::new(EmptyErrorReporter, reader);

        let res = parser.parse();

        assert_eq!(res.views.len(), 1);
        assert_eq!(res.views.values().next().unwrap().children.len(), 0);
        assert_eq!(res.templates.len(), 0);
    }

    #[test]
    fn accept_well_formed_repeat() {
        let reader = BufReader::new(
            "<view>\
                <repeat iter=\"{arf}\" template-name=\"test\"/>\
             </view>
            ".as_bytes());
        let mut parser = super::Parser::new(EmptyErrorReporter, reader);

        let res = parser.parse();

        assert_eq!(res.views.len(), 1);
        assert_eq!(res.views.values().next().unwrap().children.len(), 1);
        assert_eq!(res.templates.len(), 0);
    }
}
