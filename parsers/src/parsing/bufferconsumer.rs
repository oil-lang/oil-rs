use std::f32;
use std::io::BufRead;
use std::io::Chars;
use super::Error;


pub struct BufferConsumer<B> {
    row: usize,
    col: usize,
    buffer: Chars<B>,
    tmp_char: Option<char>,
}

impl<B> BufferConsumer<B>
    where B: BufRead
{
    pub fn new(reader: B) -> BufferConsumer<B> {
        BufferConsumer {
            row: 0,
            col: 0,
            buffer: reader.chars(),
            tmp_char: None,
        }
    }

    pub fn consume_word(&mut self) -> Result<String, Error> {
        self.consume_while(valid_alphabet_char)
    }

    pub fn consume_identifier(&mut self) -> Result<String, Error> {
        self.consume_while(valid_identifier_char)
    }

    pub fn consume_path(&mut self) -> Result<String, Error> {
        self.consume_while(valid_path_char)
    }

    pub fn consume_number(&mut self) -> Result<f32, Error> {
        let num: String = try!(self.consume_while(is_numeric));
        f32::from_str_radix(&num, 10)
            .map_err(|err| {
                Error::new(self.row, self.col, format!("Incorrect float value: {}", err))
            })
    }

    pub fn consume_whitespace(&mut self) -> Result<(), Error> {
        try!(self.consume_while(char::is_whitespace));
        Ok(())
    }

    pub fn expect_char(&mut self, expect: char) -> Result<(), Error> {
        match self.look_next_char() {
            Some(c) if c == expect => {
                self.consume_any_char();
                Ok(())
            }
            Some(c) => {
                Err(self.error_str(format!(
                    "Expected character `{}` found: `{}`",
                    expect,
                    c
                )))
            }
            _ => {
                Err(self.error_str(format!(
                    "Unexpected end of stream, expected `{}`",
                    expect
                )))
            }
        }
    }

    /// Consume characters until `test` returns false.
    /// This function return Err() only if the end of the stream
    /// is encountered.
    pub fn consume_while<F>(&mut self, test: F) -> Result<String, Error>
        where F: Fn(char) -> bool
    {
        let mut result = String::new();
        loop {
            match self.look_next_char() {
                Some(c) => {
                    if !test(c) {
                        return Ok(result)
                    }
                    self.consume_any_char();
                    result.push(c);
                }
                None => return Err(self.error("Unexpected end of stream"))
            }
        }
    }

    pub fn consume_any_char(&mut self) -> Option<char> {
        if self.tmp_char.is_none() {
            match self.buffer.next().and_then(|a| a.ok()) {
                Some(c) => {
                    if c == '\n' {
                        self.row += 1;
                        self.col = 0;
                    } else {
                        self.col += 1;
                    }
                    self.tmp_char = Some(c);
                    Some(c)
                }
                None => None
            }
        } else {
            let c = self.tmp_char.unwrap();
            self.tmp_char = None;
            Some(c)
        }
    }

    pub fn look_next_char(&mut self) -> Option<char> {
        if self.tmp_char.is_none() {
            self.consume_any_char()
        } else {
            self.tmp_char
        }
    }

    pub fn error(&self, msg: &str) -> Error {
        Error::new(self.row, self.col, msg.to_string())
    }

    pub fn error_str(&self, msg: String) -> Error {
        Error::new(self.row, self.col, msg)
    }

    pub fn error_eof(&self) -> Error {
        Error::new(self.row, self.col, "Unexpected end of stream".to_string())
    }
}

// ======================================== //
//                  HELPERS                 //
// ======================================== //

fn is_numeric(c: char) -> bool {
    char::is_numeric(c) || c == '.'
}

fn valid_alphabet_char(c: char) -> bool {
    match c {
        'a'...'z' | 'A'...'Z' => true,
        _ => false,
    }
}

fn valid_identifier_char(c: char) -> bool {
    match c {
        '0'...'9' | '-' | '_' => true,
        _ => valid_alphabet_char(c),
    }
}

fn valid_path_char(c: char) -> bool {
    match c {
        '.' => true,
        _ => valid_identifier_char(c)
    }
}
