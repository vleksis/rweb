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

impl NodeKind {
    pub(super) fn children_mut(&mut self) -> Option<&mut Vec<NodeId>> {
        match self {
            NodeKind::Document(document) => Some(&mut document.children),
            NodeKind::Tag(tag) => Some(&mut tag.children),
            NodeKind::Text(_) => None,
        }
    }
}

#[derive(Debug)]
pub(super) struct DocumentNode {
    pub(super) children: Vec<NodeId>,
}

#[derive(Debug)]
pub(super) struct TagNode {
    pub(super) tag: Tag,
    pub(super) attributes: Vec<Attribute>,
    pub(super) children: Vec<NodeId>,
}

#[derive(Debug)]
pub struct Attribute {
    pub(super) name: Range<usize>,
    pub(super) value: Option<Range<usize>>,
}

impl Attribute {
    pub fn name<'s>(&self, document: &'s Document) -> &'s str {
        &document.source[self.name.clone()]
    }

    pub fn value<'s>(&self, document: &'s Document) -> Option<&'s str> {
        self.value
            .as_ref()
            .map(|value| &document.source[value.clone()])
    }
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
        attributes: &'s [Attribute],
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
                attributes: &tag.attributes,
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
