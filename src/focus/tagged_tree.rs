use util::flat_tree::HasChildren;
use markup::NodeType;
use style::StyledNode;

pub struct TaggedNode {
    pub is_acceptor: bool,
    pub kids: Vec<TaggedNode>,
}

impl HasChildren for TaggedNode {

    fn children(& self) -> &Vec<TaggedNode> {
        &self.kids
    }
}

impl TaggedNode {

    pub fn new(node: &StyledNode) -> TaggedNode {

        let mut children = Vec::with_capacity(node.kids.len());
        // For now, the only node focus acceptor is `button`.
        let mut is_acceptor = if let NodeType::Button(_) = node.node.node_type {
            true
        } else {
            false
        };

        for kid in node.kids.iter() {
            let child = TaggedNode::new(kid);
            is_acceptor |= child.is_acceptor;
            children.push(child);
        }

        TaggedNode {
            is_acceptor: is_acceptor,
            kids: children,
        }
    }
}
