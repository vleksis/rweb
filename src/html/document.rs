use crate::html::Tag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DomId(pub(super) usize);

#[derive(Debug)]
pub(super) struct DomNode {
    pub(super) parent: Option<DomId>,
    pub(super) kind: DomKind,
}

#[derive(Debug)]
pub(super) enum DomKind {
    Document(DocumentNode),
    Tag(TagNode),
    Text(TextNode),
}

impl DomKind {
    pub(super) fn children_mut(&mut self) -> Option<&mut Vec<DomId>> {
        match self {
            DomKind::Document(document) => Some(&mut document.children),
            DomKind::Tag(tag) => Some(&mut tag.children),
            DomKind::Text(_) => None,
        }
    }
}

#[derive(Debug)]
pub(super) struct DocumentNode {
    pub(super) children: Vec<DomId>,
}

#[derive(Debug)]
pub(super) struct TagNode {
    pub(super) tag: Tag,
    pub(super) attributes: Vec<Attribute>,
    pub(super) children: Vec<DomId>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Attribute {
    pub(super) name: String,
    pub(super) value: Option<String>,
}

impl Attribute {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

#[derive(Debug)]
pub(super) struct TextNode {
    pub(super) text: String,
}

#[derive(Debug)]
pub struct Document {
    source: String,
    arena: Vec<DomNode>,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeView<'s> {
    Document {
        children: &'s [DomId],
    },
    Tag {
        tag: Tag,
        attributes: &'s [Attribute],
        children: &'s [DomId],
    },
    Text(&'s str),
}

impl Document {
    pub(super) fn from_parts(source: String, arena: Vec<DomNode>) -> Self {
        Self { source, arena }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn root(&self) -> DomId {
        DomId(0)
    }

    pub fn view(&self, id: DomId) -> NodeView<'_> {
        match &self.arena[id.0].kind {
            DomKind::Document(document) => NodeView::Document {
                children: &document.children,
            },
            DomKind::Tag(tag) => NodeView::Tag {
                tag: tag.tag,
                attributes: &tag.attributes,
                children: &tag.children,
            },
            DomKind::Text(node) => NodeView::Text(&node.text),
        }
    }

    pub fn children(&self, id: DomId) -> &[DomId] {
        match &self.arena[id.0].kind {
            DomKind::Document(document) => &document.children,
            DomKind::Tag(tag) => &tag.children,
            _ => &[],
        }
    }

    pub fn tag(&self, id: DomId) -> Option<Tag> {
        match &self.arena[id.0].kind {
            DomKind::Tag(tag) => Some(tag.tag),
            _ => None,
        }
    }

    pub fn text(&self, id: DomId) -> Option<&str> {
        match &self.arena[id.0].kind {
            DomKind::Text(node) => Some(&node.text),
            _ => None,
        }
    }

    pub fn parent(&self, id: DomId) -> Option<DomId> {
        self.arena[id.0].parent
    }
}
