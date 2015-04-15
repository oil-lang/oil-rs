
use report::ErrorReporter;
use uil_shared::deps::StyleDefinitions;
use uil_shared::deps::Constructor;
use uil_shared::asset;
use std::io::BufRead;
use std::ops::Deref;
use parsing::Error;
use parsing::BufferConsumer;
use uil_shared::resource::BasicResourceManager;
use phf;

use uil_shared::style::Value;
use uil_shared::style::KwValue;
use uil_shared::style::Rule;
use uil_shared::style::Stylesheet;
use uil_shared::style::Unit;
use uil_shared::style::Declaration;

/// Parser
pub struct Parser<'a, 'b, R: 'b, E, B> {
    err: E,
    bc: BufferConsumer<B>,
    deps: &'a StyleDefinitions,
    resource_manager: &'b mut R,
}

impl<'a, 'b, R, E, B> Parser<'a, 'b, R, E, B>
    where E: ErrorReporter,
          B: BufRead,
          R: BasicResourceManager
{

    pub fn new(
        reporter: E,
        reader: B,
        deps: &'a StyleDefinitions,
        resource_manager: &'b mut R) -> Parser<'a,'b, R, E, B>
    {
        Parser {
            bc: BufferConsumer::new(reader),
            err: reporter,
            deps: deps,
            resource_manager: resource_manager,
        }
    }

    pub fn parse(&mut self) -> Stylesheet {

        // Create stylesheet
        let mut stylesheet = Stylesheet::new();

        // Read token from the buffer.
        'rule: loop {
            match self.bc.consume_whitespace() {
                Ok(_) => (),
                _ => {
                    break 'rule;
                }
            }

            // Is there anything to read ?
            match self.bc.look_next_char() {
                None => break 'rule,
                _ => ()
            }

            match self.parse_rule() {
                Ok(rule) => {
                    stylesheet.rules.push(rule);
                }
                Err(err) => {
                    self.err.log(format!("Error {}", err));
                    break 'rule;
                }
            }
        }

        stylesheet
    }

    fn parse_rule(&mut self) -> Result<Rule, Error> {

        let selector = try!(self.parse_selector());
        let mut declarations = Vec::new();

        try!(self.bc.consume_whitespace());
        match self.bc.consume_any_char() {
            Some('{') => (),
            _ => return Err(self.bc.error("Rule must start with a `{`"))
        }


        // Loop for declaration.
        'decl: loop {
            try!(self.bc.consume_whitespace());

            match self.bc.look_next_char() {
                Some('}') => break 'decl,
                Some(_) => {
                    let decl =  try!(self.parse_declaration());
                    declarations.push(decl);
                }
                None => return Err(self.bc.error("Selector must end with a `}`"))
            }
        }

        // Consume '}'
        self.bc.consume_any_char().unwrap();

        Ok(Rule {
            selector: selector,
            declarations: declarations
        })
    }

    fn parse_selector(&mut self) -> Result<String, Error> {

        try!(self.bc.consume_whitespace());
        match self.bc.consume_any_char() {
            Some('.') => (),
            _ => return Err(self.bc.error("Selector must start with a `.`"))
        }
        self.bc.consume_identifier()
    }

    fn parse_declaration(&mut self) -> Result<Declaration, Error> {

        try!(self.bc.consume_whitespace());

        let name = try!(self.bc.consume_identifier());

        try!(self.bc.consume_whitespace());
        match self.bc.consume_any_char() {
            Some(':') => (),
            _ => return Err(self.bc.error("Invalid identifier expected `:`"))
        }

        let value = try!(self.parse_value());

        try!(self.bc.consume_whitespace());
        match self.bc.consume_any_char() {
            Some(';') => (),
            _ => return Err(self.bc.error("Declaration should end with `;`"))
        }

        Ok(Declaration {
            name: name,
            value: value
        })
    }

    fn parse_value(&mut self) -> Result<Value, Error> {

        try!(self.bc.consume_whitespace());
        match self.bc.look_next_char() {
            Some(c) => match c {
                '$' => {
                    self.bc.consume_any_char();
                    let path = try!(self.bc.consume_path());
                    match self.deps.defs.get(&path) {
                        Some(v) => {
                            if let Some(val) = convert_to_style_value(v, self.resource_manager) {
                                Ok(val)
                            } else {
                                Err(self.bc.error_str(
                                    format!("Resource `{}` failed loading.", path)
                                ))
                            }
                        }
                        None => Err(self.bc.error_str(
                            format!("Couldn't find `{}` in style definitions", path)
                        ))
                    }
                },
                '0'...'9' => {
                    let val = try!(self.bc.consume_number());
                    let unit = try!(self.consume_unit());
                    Ok(Value::Length(val, unit))
                }
                _ => {
                    let keyword = try!(self.bc.consume_identifier());
                    if let Some(&k) = KEYWORDS.get(keyword.deref()) {
                        Ok(Value::Keyword(k))
                    } else {
                        Err(self.bc.error_str(
                            format!("Unknown keyword: `{}`", keyword)
                        ))
                    }
                }
            },
            None => Err(self.bc.error("Unexpected end of input. Expected Value."))
        }
    }

    fn consume_unit(&mut self) -> Result<Unit, Error> {
        try!(self.bc.consume_identifier());
        Ok(Unit::Px)
    }
}

static KEYWORDS: phf::Map<&'static str, KwValue> = phf_map! {
    "auto" => KwValue::Auto,
    "expand" => KwValue::Expand,
    "absolute" => KwValue::Absolute,
    "fit" => KwValue::Fit,
    "repeat" => KwValue::Repeat
};

fn convert_to_style_value<R>(ctor: &Constructor, resource_manager: &mut R)
    -> Option<Value>
    where R: BasicResourceManager
{
    match *ctor {
        Constructor::Number(v) => Some(Value::Length(v, Unit::Px)),
        Constructor::Quote(ref q) => match KEYWORDS.get(q.deref()) {
            Some(&k) => Some(Value::Keyword(k)),
            _ => None
        },
        Constructor::Font(..) => Some(Value::Font(asset::FontData::new(ctor))),
        Constructor::Image(..) => Some(Value::Image(asset::ImageData::new(ctor, resource_manager))),
        Constructor::None => None,
    }
}
