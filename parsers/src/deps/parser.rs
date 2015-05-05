
use std::io::BufRead;
use std::ops::Deref;
use std::path::{PathBuf, Path};
use report::ErrorReporter;
use parsing::BufferConsumer;
use parsing::Error;

use oil_shared::deps::Constructor;
use oil_shared::deps::StyleDefinitions;

pub struct Parser<E, B> {
    err: E,
    bc: BufferConsumer<B>,
    prefix: String,
    relative_to: PathBuf,
}


impl<E, B> Parser<E, B>
    where E: ErrorReporter,
          B: BufRead
{

    pub fn new(reporter: E, reader: B, folder_parent: PathBuf) -> Parser<E, B> {
        Parser {
            err: reporter,
            bc: BufferConsumer::new(reader),
            prefix: "".to_string(),
            relative_to: folder_parent,
        }
    }

    pub fn parse(&mut self) -> StyleDefinitions {
        let mut styledefs = StyleDefinitions::new();

        // Read token from the buffer.
        'def: loop {
            match self.bc.consume_whitespace() {
                Ok(_) => (),
                _ => {
                    break 'def;
                }
            }

            // Is there anything to read ?
            match self.bc.look_next_char() {
                None       => break 'def,
                Some('[')  => match self.parse_prefix() {
                    Ok(prefix) => self.prefix = prefix,
                    Err(err) => {
                        self.err.log(format!("Error {}", err));
                        break 'def;
                    }
                },
                _          => match self.parse_def() {
                    Ok((name, value)) => {
                        if self.prefix.is_empty() {
                            styledefs.insert(name, value);
                        } else {
                            styledefs.insert(
                                "".to_string() + &self.prefix + &"." + &name,
                                value);
                        }
                    }
                    Err(err) => {
                        self.err.log(format!("Error {}", err));
                        break 'def;
                    }
                }
            }
        }


        styledefs
    }

    fn parse_prefix(&mut self) -> Result<String, Error> {
        self.bc.consume_any_char();
        try!(self.bc.consume_whitespace());
        let prefix = try!(self.bc.consume_identifier());
        try!(self.bc.consume_whitespace());
        try!(self.bc.expect_char(']'));
        Ok(prefix)
    }

    fn parse_def(&mut self) -> Result<(String, Constructor), Error> {
        let name = try!(self.bc.consume_path());
        try!(self.bc.consume_whitespace());
        try!(self.bc.expect_char('='));
        try!(self.bc.consume_whitespace());
        let value = try!(self.parse_value());
        Ok((name, value))
    }

    fn parse_value(&mut self) -> Result<Constructor, Error> {

        let c = match self.bc.look_next_char() {
            Some(c) => c,
            None => return Err(self.bc.error_eof())
        };

        match c {
            '"' => self.parse_quote(),
            '0'...'9' => self.parse_number(),
            _ => self.parse_ctor()
        }
    }

    fn parse_quote(&mut self) -> Result<Constructor, Error> {
        Ok(Constructor::Quote(try!(self.consume_quote())))
    }

    fn parse_number(&mut self) -> Result<Constructor, Error> {
        Ok(Constructor::Number(try!(self.bc.consume_number())))
    }

    fn parse_ctor(&mut self) -> Result<Constructor, Error> {
        let ctor = try!(self.bc.consume_word());
        try!(self.bc.consume_whitespace());
        let args = try!(self.parse_args());

        match ctor.deref() {
            "Font" => {
                let path = try!(self.find_str_arg(args.iter(), "path", 0));
                let width = try!(self.find_num_arg(args.iter(), "width", 0));
                let height = try!(self.find_num_arg(args.iter(), "height", 1));
                Ok(Constructor::Font(path, width, height))
            },
            "Image" => {
                let path = try!(self.find_str_arg(args.iter(), "path", 0));
                let width = self.find_num_arg(args.iter(), "width", 0).ok();
                let height = self.find_num_arg(args.iter(), "height", 1).ok();
                let offset_x = self.find_num_arg(args.iter(), "offset-x", 2).ok();
                let offset_y = self.find_num_arg(args.iter(), "offset-y", 3).ok();
                Ok(Constructor::Image(self.resolve_path(path), width, height, offset_x, offset_y))
            }
            _ => {
                Err(self.bc.error(
                    "Unknown constructor. \
                    Can be either `Image` or `Font`"
                ))
            }
        }
    }

    fn resolve_path(&self, path: String) -> PathBuf {
        self.relative_to.join(Path::new(&path))
    }

    fn parse_args(&mut self) -> Result<Vec<Arg>, Error> {

        fn is_separator(c: char) -> bool {
            c.is_whitespace() || c == ','
        }
        try!(self.bc.expect_char('('));

        let mut args = Vec::new();

        'args: loop {
            try!(self.bc.consume_while(is_separator));
            let c = match self.bc.look_next_char() {
                Some(c) => c,
                None => return Err(self.bc.error_eof())
            };

            match c {
                ')' => {
                    break 'args;
                },
                _ => {
                    args.push(try!(self.parse_one_arg()));
                }
            }
        }
        // Consume ')'
        self.bc.consume_any_char().unwrap();
        Ok(args)
    }

    fn parse_one_arg(&mut self) -> Result<Arg, Error> {
        try!(self.bc.consume_whitespace());
        let c = match self.bc.look_next_char() {
            Some(c) => c,
            None => return Err(self.bc.error_eof())
        };

        match c {
            '"' => {
                let val = try!(self.consume_quote());
                Ok(Arg {
                    name: "".to_string(),
                    arg_type: ArgType::Strstr(val)
                })
            }
            '0'...'9' => {
                let val = try!(self.bc.consume_number());
                Ok(Arg {
                    name: "".to_string(),
                    arg_type: ArgType::Number(val)
                })
            }
            _ => {
                let name = try!(self.bc.consume_identifier());
                try!(self.bc.consume_whitespace());
                try!(self.bc.expect_char(':'));
                try!(self.bc.consume_whitespace());

                let c = match self.bc.look_next_char() {
                    Some(c) => c,
                    None => return Err(self.bc.error_eof())
                };

                match c {
                    '"' => {
                        let val = try!(self.consume_quote());
                        Ok(Arg {
                            name: name,
                            arg_type: ArgType::Strstr(val)
                        })
                    }
                    '0'...'9' => {
                        let val = try!(self.bc.consume_number());
                        Ok(Arg {
                            name: name,
                            arg_type: ArgType::Number(val)
                        })
                    }
                    _ => Err(self.bc.error(
                        "Unknown argument type. \
                        Can be either `String` or `Number`"
                    ))
                }
            }
        }
    }

    fn consume_quote(&mut self) -> Result<String, Error> {
        try!(self.bc.expect_char('"'));
        let quote = try!(self.bc.consume_while(|c| c != '"'));
        try!(self.bc.expect_char('"'));
        Ok(quote)
    }

    fn find_str_arg<'a, I>(&self, iter: I, name: &str, pos: usize)
        -> Result<String, Error>
        where I: Iterator<Item=&'a Arg> + Clone
    {
        let i1 = iter.clone().filter(|&x| {
            match x.arg_type {
                ArgType::Strstr(_) => true,
                _ => false
            }
        });
        let i2 = iter.clone().filter(|&x| {
            match x.arg_type {
                ArgType::Strstr(_) => true,
                _ => false
            }
        });

        self.find_type_arg(name, pos, i1, i2).map(|arg| {
            match arg.arg_type {
                ArgType::Strstr(ref val) => val.clone(),
                _ => unreachable!()
            }
        })
    }

    fn find_num_arg<'a, I>(&self, iter: I, name: &str, pos: usize)
        -> Result<f32, Error>
        where I: Iterator<Item=&'a Arg> + Clone
    {
        let i1 = iter.clone().filter(|&x| {
            match x.arg_type {
                ArgType::Number(_) => true,
                _ => false
            }
        });
        let i2 = iter.clone().filter(|&x| {
            match x.arg_type {
                ArgType::Number(_) => true,
                _ => false
            }
        });

        self.find_type_arg(name, pos, i1, i2).map(|arg| {
            match arg.arg_type {
                ArgType::Number(val) => val,
                _ => unreachable!()
            }
        })
    }

    fn find_type_arg<'a, I1, I2>(
        &self,
        name: &str,
        pos:usize,
        mut bt1: I1,
        mut bt2: I2) -> Result<&'a Arg, Error>
        where I1: Iterator<Item=&'a Arg>,
              I2: Iterator<Item=&'a Arg>
    {
        let try_by_name = bt1.find(|&x| {
            x.name == name
        });

        let by_pos = bt2.nth(pos);

        if try_by_name.is_none() {
            match by_pos {
                Some(a) => Ok(a),
                None => Err(self.bc.error_str(
                    format!("argument `{}` not found", name
                )))
            }
        } else {
            Ok(try_by_name.unwrap())
        }
    }

}

#[derive(Clone, Debug)]
enum ArgType {
    Strstr(String),
    Number(f32)
}

#[derive(Clone, Debug)]
struct Arg {
    pub name: String,
    pub arg_type: ArgType,
}
