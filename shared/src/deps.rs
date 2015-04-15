use std::collections::HashMap;
use std::path::PathBuf;

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
