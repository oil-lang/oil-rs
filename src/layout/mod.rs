
pub use self::boxes::LayoutBuffer;
pub use self::boxes::LayoutBox;
pub use self::dim::Dimensions;
pub use self::dim::EdgeSizes;

mod boxes;
mod dim;

pub trait Box {
    fn dimensions(&self) -> Dimensions;
}
