
pub use self::dim::Dimensions;
pub use self::dim::EdgeSizes;
pub use self::dim::Rect;
pub use self::boxes::LayoutBuffer;
pub use self::boxes::LayoutBox;

mod boxes;
mod dim;

#[cfg(test)]
mod test {
    // TODO
}
