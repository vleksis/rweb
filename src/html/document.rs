use std::ops::Range;

use crate::html::Tag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub(super) usize);

#[derive(Debug)]
pub(super) struct Node {
    pub(super) parent: Option<NodeId>,
    pub(super) kind: NodeKind,
}

#[derive(Debug)]
pub(super) enum NodeKind {
    Document(DocumentNode),
    Tag(TagNode),
    Text(TextNode),
}

#[derive(Debug)]
pub(super) struct DocumentNode {
    pub(super) children: Vec<NodeId>,
}

#[derive(Debug)]
pub(super) struct TagNode {
    pub(super) tag: Tag,
    pub(super) attributes: Range<usize>,
    pub(super) children: Vec<NodeId>,
}

#[derive(Debug)]
pub(super) struct TextNode {
    pub(super) range: Range<usize>,
}

#[derive(Debug)]
pub struct Document {
    source: String,
    arena: Vec<Node>,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeView<'s> {
    Document {
        children: &'s [NodeId],
    },
    Tag {
        tag: Tag,
        attributes: &'s str,
        children: &'s [NodeId],
    },
    Text(&'s str),
}

impl Document {
    pub(super) fn from_parts(source: String, arena: Vec<Node>) -> Self {
        Self { source, arena }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn root(&self) -> NodeId {
        NodeId(0)
    }

    pub fn view(&self, id: NodeId) -> NodeView<'_> {
        match &self.arena[id.0].kind {
            NodeKind::Document(document) => NodeView::Document {
                children: &document.children,
            },
            NodeKind::Tag(tag) => NodeView::Tag {
                tag: tag.tag,
                attributes: &self.source[tag.attributes.clone()],
                children: &tag.children,
            },
            NodeKind::Text(text) => NodeView::Text(&self.source[text.range.clone()]),
        }
    }

    pub fn children(&self, id: NodeId) -> &[NodeId] {
        match &self.arena[id.0].kind {
            NodeKind::Document(document) => &document.children,
            NodeKind::Tag(tag) => &tag.children,
            _ => &[],
        }
    }

    pub fn tag(&self, id: NodeId) -> Option<Tag> {
        match &self.arena[id.0].kind {
            NodeKind::Tag(tag) => Some(tag.tag),
            _ => None,
        }
    }

    pub fn text(&self, id: NodeId) -> Option<&str> {
        match &self.arena[id.0].kind {
            NodeKind::Text(text) => Some(&self.source[text.range.clone()]),
            _ => None,
        }
    }

    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.arena[id.0].parent
    }
}
