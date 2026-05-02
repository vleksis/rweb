use std::ops::Range;

use crate::html::document::Document;
use crate::html::document::DocumentNode;
use crate::html::document::Node;
use crate::html::document::NodeId;
use crate::html::document::NodeKind;
use crate::html::document::TagNode;
use crate::html::document::TextNode;
use crate::html::tag::Tag;
use crate::html::tag::TagKind;

struct Parser {
    pos: usize,
    builder: DocumentBuilder,
}

impl Parser {
    fn new(source: String) -> Self {
        Self {
            pos: 0,
            builder: DocumentBuilder::new(source),
        }
    }

    fn source(&self) -> &str {
        &self.builder.source
    }

    fn peek(&self) -> Option<char> {
        self.source()[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        match self.source()[self.pos..].chars().next() {
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

    fn current_parent(&self) -> NodeId {
        self.builder.current_parent()
    }

    fn push_tag(&mut self, tag: ParsingTag) {
        self.builder.push_tag(tag);
    }

    fn push_text(&mut self, range: Range<usize>) {
        self.builder.push_text(self.current_parent(), range);
    }

    fn finish(self) -> Document {
        self.builder.build()
    }
}

#[derive(Debug)]
struct DocumentBuilder {
    source: String,
    arena: Vec<Node>,
    unfinished: Vec<(Tag, NodeId)>,
}

impl DocumentBuilder {
    fn new(source: String) -> Self {
        let root = Node {
            parent: None,
            kind: NodeKind::Document(DocumentNode {
                children: Vec::new(),
            }),
        };

        Self {
            source,
            arena: vec![root],
            unfinished: Vec::new(),
        }
    }

    fn current_parent(&self) -> NodeId {
        self.unfinished
            .last()
            .map(|(_, id)| *id)
            .unwrap_or_else(|| NodeId(0))
    }

    fn push_tag(&mut self, tag: ParsingTag) {
        match tag.kind {
            TagKind::Open => {
                let id = self.push_tag_node(self.current_parent(), tag.tag, tag.attributes);
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
                self.push_tag_node(self.current_parent(), tag.tag, tag.attributes);
            }
        };
    }

    fn push_tag_node(&mut self, parent: NodeId, tag: Tag, attributes: Range<usize>) -> NodeId {
        self.push_node(
            parent,
            NodeKind::Tag(TagNode {
                tag,
                attributes,
                children: Vec::new(),
            }),
        )
    }

    fn push_text(&mut self, parent: NodeId, range: Range<usize>) -> NodeId {
        self.push_node(parent, NodeKind::Text(TextNode { range }))
    }

    fn push_node(&mut self, parent: NodeId, kind: NodeKind) -> NodeId {
        let id = NodeId(self.arena.len());

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

    fn build(self) -> Document {
        Document::from_parts(self.source, self.arena)
    }
}

pub fn parse(source: String) -> Document {
    let mut parser = Parser::new(source);

    while let Some(c) = parser.peek() {
        if c == '<' {
            let tag = parse_tag(&mut parser);
            parser.push_tag(tag);
        } else {
            let text = parse_text(&mut parser);
            parser.push_text(text);
        }
    }

    parser.finish()
}

fn parse_tag(parser: &mut Parser) -> ParsingTag {
    parser.consume('<');

    let start = parser.pos;

    loop {
        let Some(c) = parser.peek() else {
            panic!("Tag is not closed: {}", &parser.source()[parser.pos..]);
        };

        if c == '>' {
            parser.consume('>');
            break;
        }

        parser.advance();
    }

    let mut raw_start = start;
    let mut raw_end = parser.pos - 1;
    let source = parser.source();

    while raw_start < raw_end && source[raw_start..].starts_with(char::is_whitespace) {
        raw_start += source[raw_start..].chars().next().unwrap().len_utf8();
    }
    while raw_end > raw_start && source[..raw_end].ends_with(char::is_whitespace) {
        raw_end -= source[..raw_end].chars().next_back().unwrap().len_utf8();
    }

    let closing = source[raw_start..raw_end].starts_with('/');
    if closing {
        raw_start += 1;
        while raw_start < raw_end && source[raw_start..].starts_with(char::is_whitespace) {
            raw_start += source[raw_start..].chars().next().unwrap().len_utf8();
        }
    }

    let explicit_self_closing = source[raw_start..raw_end].ends_with('/');
    if explicit_self_closing {
        raw_end -= 1;
        while raw_end > raw_start && source[..raw_end].ends_with(char::is_whitespace) {
            raw_end -= source[..raw_end].chars().next_back().unwrap().len_utf8();
        }
    }

    let name_end = source[raw_start..raw_end]
        .find(char::is_whitespace)
        .map(|offset| raw_start + offset)
        .unwrap_or(raw_end);
    let name = source[raw_start..name_end].to_ascii_lowercase();
    let mut attributes_start = name_end;
    while attributes_start < raw_end && source[attributes_start..].starts_with(char::is_whitespace)
    {
        attributes_start += source[attributes_start..]
            .chars()
            .next()
            .unwrap()
            .len_utf8();
    }
    let attributes = attributes_start..raw_end;

    let tag = Tag::parse(&name);

    if closing {
        ParsingTag {
            tag,
            attributes,
            kind: TagKind::Close,
        }
    } else if explicit_self_closing || tag.is_self_closing() || name.starts_with('!') {
        ParsingTag {
            tag,
            attributes,
            kind: TagKind::SelfClosing,
        }
    } else {
        ParsingTag {
            tag,
            attributes,
            kind: TagKind::Open,
        }
    }
}

fn parse_text(parser: &mut Parser) -> Range<usize> {
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

    start..parser.pos
}

#[derive(Debug)]
struct ParsingTag {
    tag: Tag,
    attributes: Range<usize>,
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
    fn stores_raw_attributes() {
        let document = parse("<a href=http://example.org class=external>link</a>".to_string());
        let link = document.children(document.root())[0];

        let attributes = match document.view(link) {
            crate::html::NodeView::Tag { attributes, .. } => attributes,
            _ => panic!("expected tag node"),
        };

        assert_eq!(document.tag(link), Some(Tag::A));
        assert_eq!(attributes, "href=http://example.org class=external");
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
