
mod parser;

// Dependencies
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{PathBuf, Path};
use std::fs::File;

use report::ErrorReporter;
use asset;

/// Convenient function to parse a style from a BufRead.
///
/// This function assume that the external dependencies,
/// if any, are to be found in the current working directory.
///
/// If you have an external file, prefer using: parse_file for now.
///
/// ## Panics
///
/// This function never panic if the style is not properly defined.
/// However you'll get an uncomplete StyleDefinitions that might
/// result in a subsequent panic.
///
/// If the reporter is set to StdOutReporter, then you'll have a detail
/// explanation of the error encountered.
pub fn parse<E, B>(reporter: E, reader: B) -> StyleDefinitions
    where E: ErrorReporter,
          B: BufRead
{
    let mut parser = parser::Parser::new(
        reporter,
        reader,
        Path::new(".").to_path_buf()
    );
    parser.parse()
}

/// Parse a given style file and return the StyleDefinitions.
///
/// This function is strictly equivalent to the above one, except that it
/// use the parent directory to the file to find the external dependencies.
///
/// ## Panics
///
/// This function panics if the file can't be found.
/// Otherwise see detailed explanation of `parse()`.
pub fn parse_file<E, P>(reporter: E, path: P) -> StyleDefinitions
    where E: ErrorReporter,
          P: AsRef<Path>
{
    let reader = BufReader::new(File::open(path.as_ref()).unwrap());
    let mut parser = parser::Parser::new(
        reporter,
        reader,
        path.as_ref().parent().unwrap_or(Path::new(".")).to_path_buf()
    );
    parser.parse()
}

pub struct StyleDefinitions {
    pub defs: HashMap<String, Constructor>,
}

impl StyleDefinitions {
    pub fn new() -> StyleDefinitions {
        StyleDefinitions {
            defs: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, ctor: Constructor) {
        self.defs.insert(key, ctor);
    }
}

#[derive(Clone, Debug)]
pub enum Constructor {
    /// None type -> Constructor failed loading the resource.
    None,
    /// Number [0-9]+
    Number(f32),
    /// String ".+"
    Quote(String),
    /// Font(path, width, height)
    Font(String, f32, f32),
    /// TODO: replace String by the type Path
    /// Image(path, width, height, offset-x, offset-y)
    Image(PathBuf, Option<f32>, Option<f32>, Option<f32>, Option<f32>),
    // Add other construtor here...
}
