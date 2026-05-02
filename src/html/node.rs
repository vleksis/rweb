use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Range;

use crate::html::tag::Tag;
use crate::html::tag::TagKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

#[derive(Debug)]
pub struct Node {
    parent: Option<NodeId>,
    kind: NodeKind,
}

#[derive(Debug)]
enum NodeKind {
    Document(DocumentNode),
    Tag(TagNode),
    Text(TextNode),
}

#[derive(Debug)]
struct DocumentNode {
    children: Vec<NodeId>,
}

#[derive(Debug)]
struct TagNode {
    tag: Tag,
    children: Vec<NodeId>,
}

#[derive(Debug)]
struct TextNode {
    range: Range<usize>,
}

#[derive(Debug)]
pub struct Document {
    source: String,
    arena: Vec<Node>,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeView<'s> {
    Document { children: &'s [NodeId] },
    Tag { tag: Tag, children: &'s [NodeId] },
    Text(&'s str),
}

impl Document {
    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn root(&self) -> NodeId {
        NodeId(0)
    }

    pub fn view(&self, id: NodeId) -> NodeView<'_> {
        match &self[id].kind {
            NodeKind::Document(document) => NodeView::Document {
                children: &document.children,
            },
            NodeKind::Tag(tag) => NodeView::Tag {
                tag: tag.tag,
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
        match &self[id].kind {
            NodeKind::Tag(tag) => Some(tag.tag),
            _ => None,
        }
    }

    pub fn text(&self, id: NodeId) -> Option<&str> {
        match &self[id].kind {
            NodeKind::Text(text) => Some(&self.source[text.range.clone()]),
            _ => None,
        }
    }

    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self[id].parent
    }
}

impl Index<NodeId> for Document {
    type Output = Node;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.arena[index.0]
    }
}

impl IndexMut<NodeId> for Document {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.arena[index.0]
    }
}

#[derive(Debug)]
pub struct DocumentBuilder {
    arena: Vec<Node>,
    unfinished: Vec<(Tag, NodeId)>,
}

impl DocumentBuilder {
    fn new() -> Self {
        let root = Node {
            parent: None,
            kind: NodeKind::Document(DocumentNode {
                children: Vec::new(),
            }),
        };
        let arena = vec![root];

        Self {
            arena,
            unfinished: Vec::new(),
        }
    }

    fn push_node(&mut self, kind: NodeKind) -> NodeId {
        let id = NodeId(self.arena.len());
        let parent = self.unfinished.last().map(|x| x.1).unwrap_or(NodeId(0));

        match &mut self.arena[parent.0].kind {
            NodeKind::Document(document) => document.children.push(id),
            NodeKind::Tag(tag) => tag.children.push(id),
            NodeKind::Text(_) => unreachable!(),
        };

        self.arena.push(Node {
            parent: Some(parent),
            kind,
        });
        id
    }

    fn push_tag(&mut self, tag: ParsingTag) {
        match tag.kind {
            TagKind::Open => {
                let kind = NodeKind::Tag(TagNode {
                    tag: tag.tag,
                    children: Vec::new(),
                });
                let id = self.push_node(kind);
                self.unfinished.push((tag.tag, id));
            }

            TagKind::Close => match self.unfinished.last() {
                Some((last, _)) => {
                    if *last != tag.tag {
                        panic!("mismatched tag: expected {:?}, got {:?}", last, tag.tag);
                    }

                    self.unfinished.pop();
                }
                None => panic!("unmatched close tag: {:?}", tag.tag),
            },

            TagKind::SelfClosing => {
                let kind = NodeKind::Tag(TagNode {
                    tag: tag.tag,
                    children: Vec::new(),
                });
                self.push_node(kind);
            }
        };
    }

    fn push_text(&mut self, text: TextNode) {
        let kind = NodeKind::Text(text);
        self.push_node(kind);
    }
}

struct Parser<'s> {
    source: &'s str,
    pos: usize,
    builder: DocumentBuilder,
}

impl<'s> Parser<'s> {
    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        match self.source[self.pos..].chars().next() {
            Some(c) => {
                self.pos += c.len_utf8();
                Some(c)
            }

            None => None,
        }
    }

    fn consume(&mut self, expected: char) {
        if self.peek() != Some(expected) {
            panic!(
                "Expected '{}', got '{}'",
                expected,
                self.peek().unwrap_or('?')
            );
        }

        self.advance();
    }
}

pub fn parse(source: String) -> Document {
    let mut parser = Parser {
        source: &source,
        pos: 0,
        builder: DocumentBuilder::new(),
    };

    while let Some(c) = parser.peek() {
        if c == '<' {
            let tag = parse_tag(&mut parser);
            parser.builder.push_tag(tag);
        } else {
            let text = parse_text(&mut parser);
            parser.builder.push_text(text);
        }
    }

    let DocumentBuilder {
        arena,
        unfinished: _,
    } = parser.builder;

    Document { source, arena }
}

fn parse_tag(parser: &mut Parser) -> ParsingTag {
    parser.consume('<');

    let start = parser.pos;

    loop {
        let Some(c) = parser.peek() else {
            panic!("Tag is not closed: {}", &parser.source[parser.pos..]);
        };

        if c == '>' {
            parser.consume('>');
            break;
        }

        parser.advance();
    }

    let mut raw = parser.source[start..parser.pos - 1].trim();
    let closing = raw.starts_with('/');
    if closing {
        raw = raw[1..].trim_start();
    }

    let explicit_self_closing = raw.ends_with('/');
    if explicit_self_closing {
        raw = raw[..raw.len() - 1].trim_end();
    }

    let name = raw
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let tag = Tag::parse(&name);

    if closing {
        ParsingTag {
            tag,
            kind: TagKind::Close,
        }
    } else if explicit_self_closing || tag.is_self_closing() || name.starts_with('!') {
        ParsingTag {
            tag,
            kind: TagKind::SelfClosing,
        }
    } else {
        ParsingTag {
            tag,
            kind: TagKind::Open,
        }
    }
}

fn parse_text(parser: &mut Parser) -> TextNode {
    let start = parser.pos;

    loop {
        let Some(c) = parser.peek() else {
            break;
        };

        if c == '<' {
            break;
        }

        parser.advance();
    }

    TextNode {
        range: Range {
            start,
            end: parser.pos,
        },
    }
}

#[derive(Debug)]
struct ParsingTag {
    tag: Tag,
    kind: TagKind,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_text_under_document_root() {
        let document = parse("hello".to_string());
        let text = document.children(document.root())[0];

        assert_eq!(document.text(text), Some("hello"));
        assert_eq!(document.parent(text), Some(document.root()));
    }

    #[test]
    fn parses_nested_tags() {
        let document = parse("<p>Hello <em>world</em></p>".to_string());
        let paragraph = document.children(document.root())[0];
        let emphasis = document.children(paragraph)[1];
        let text = document.children(emphasis)[0];

        assert_eq!(document.tag(paragraph), Some(Tag::P));
        assert_eq!(document.tag(emphasis), Some(Tag::Em));
        assert_eq!(document.text(text), Some("world"));
    }

    #[test]
    fn treats_void_tags_as_self_closing() {
        let document = parse("<p>a<br>b</p>".to_string());
        let paragraph = document.children(document.root())[0];
        let children = document.children(paragraph);

        assert_eq!(document.text(children[0]), Some("a"));
        assert_eq!(document.tag(children[1]), Some(Tag::Br));
        assert_eq!(document.text(children[2]), Some("b"));
    }

    #[test]
    fn auto_closes_unfinished_tags_at_eof() {
        let document = parse("<p>hello".to_string());
        let paragraph = document.children(document.root())[0];
        let text = document.children(paragraph)[0];

        assert_eq!(document.tag(paragraph), Some(Tag::P));
        assert_eq!(document.text(text), Some("hello"));
    }
}
