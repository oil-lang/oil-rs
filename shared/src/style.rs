
use asset;
//use color::alpha::Rgba;

#[derive(Debug)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug)]
pub struct Rule {
    pub selector: Selector,
    pub declarations: Vec<Declaration>,
}

#[derive(Copy, Clone, Debug)]
pub enum SelectorState {
    Default,
    Focus,
    Hover,
    Creation
}

#[derive(Debug)]
pub struct Selector {
    pub name: String,
    pub state: SelectorState
}

#[derive(Debug)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

// TODO: FIXME
// Remember to have a property layout to either
// render right to left (rtl) or left to right (ltr)
// ```
//      layout: rtl; // ltr is default
// ```
// That does not affect the layout algorithm
// instead, the x property is inversed as parent.width - x
// Note: KeywordAuto, KeywordAbsolute, etc ... should be merged
// at this point into Keyword(String). Leaving this for later.
//
#[derive(Debug, Clone)]
pub enum Value {
    Length(f32, Unit),
//    ColorValue(Rgba<u8>)
    Font(asset::FontData),
    Image(asset::ImageData),
    Keyword(KwValue),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum KwValue {
    Auto,
    Expand,
    Absolute,
    Fit,
    Repeat
}

#[derive(Debug, Clone)]
pub enum Unit {
    Px,
}

impl Stylesheet {

    #[inline]
    pub fn new() -> Stylesheet {
        Stylesheet {
            rules: Vec::new()
        }
    }
}
