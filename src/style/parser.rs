
use report::ErrorReporter;
use deps::StyleDefinitions;
use std::io::BufRead;
use parsing::Error;
use parsing::BufferConsumer;

use super::Value;
use super::Rule;
use super::Stylesheet;
use super::Unit;
use super::Declaration;

/// Parser
pub struct Parser<'a, E, B> {
    err: E,
    bc: BufferConsumer<B>,
    deps: &'a StyleDefinitions,
}

impl<'a, E, B> Parser<'a, E, B>
    where E: ErrorReporter,
          B: BufRead
{

    pub fn new(reporter: E, reader: B, deps: &'a StyleDefinitions) -> Parser<'a, E, B>
    {
        Parser {
            bc: BufferConsumer::new(reader),
            err: reporter,
            deps: deps,
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
                            if let Some(val) = v.convert_to_style_value() {
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
                // TODO: Fix this: Keyword should
                // be handled in a more generic way.
                'a' => {
                    let keyword = try!(self.bc.consume_identifier());
                    if keyword == "auto" {
                        Ok(Value::KeywordAuto)
                    } else if keyword == "absolute" {
                        Ok(Value::KeywordAbsolute)
                    } else {
                        Err(self.bc.error("Did you mean `auto` or `absolute`?"))
                    }
                }
                'f' => {
                    let keyword = try!(self.bc.consume_identifier());
                    if keyword == "fit" {
                        Ok(Value::KeywordFit)
                    } else {
                        Err(self.bc.error("Did you mean `fit`?"))
                    }
                }
                'r' => {
                    let keyword = try!(self.bc.consume_identifier());
                    if keyword == "repeat" {
                        Ok(Value::KeywordRepeat)
                    } else {
                        Err(self.bc.error("Did you mean `repeat`?"))
                    }
                }
                '0'...'9' => {
                    let val = try!(self.bc.consume_number());
                    let unit = try!(self.consume_unit());
                    Ok(Value::Length(val, unit))
                }
                _ => Err(self.bc.error("Unknown value."))
            },
            None => Err(self.bc.error("Unexpected end of input. Expected Value."))
        }
    }

    fn consume_unit(&mut self) -> Result<Unit, Error> {
        try!(self.bc.consume_identifier());
        Ok(Unit::Px)
    }
}
