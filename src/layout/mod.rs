
pub use self::buffer::LayoutBuffer;
pub use self::boxes::LayoutBox;
pub use self::dim::Dimensions;
pub use self::dim::EdgeSizes;

mod boxes;
mod buffer;
mod dim;

#[cfg(test)]
mod test {

    use std::old_io::BufferedReader;
    use markup::Node;
    use markup;
    use style::StyledNode;
    use style::Stylesheet;
    use style;
    use report;
    use deps::StyleDefinitions;

    fn stylesheet(st: &str) -> Stylesheet {
        let reader = BufferedReader::new(st.as_bytes());
        let defs = StyleDefinitions::new();
        style::parse(report::StdOutErrorReporter, reader, &defs)
    }

    fn markup_tree(mk: &str) -> markup::Node {
        let reader = BufferedReader::new(mk.as_bytes());
        let lib = markup::parse(report::StdOutErrorReporter, reader);
        let (_, root) = lib.views.into_iter().next().unwrap();
        root
    }

    #[test]
    fn compute_layout_one_node() {
        let stylesheet = stylesheet(
            ".btn {
                margin: auto;
            }
            ");
        let root = markup_tree(
            "<view>\
                <button class=\"btn\"></button>\
                <button class=\"btn\"></button>\
                <button class=\"btn\"></button>\
            </view>
            ");
        let StyledNode = style::build_style_tree(&root, &stylesheet);
    }
}
