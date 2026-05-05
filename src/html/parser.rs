use std::ops::Range;

use anyhow::bail;

use crate::html::document::Attribute;
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
        self.nth(0)
    }

    /// Support no more than 3 chars lookahead
    ///
    /// # Panics
    ///
    /// Panics if `n` is greater than 3.
    fn nth(&self, n: usize) -> Option<char> {
        assert!(n <= 3);
        self.remaining().chars().nth(n)
    }

    fn slice(&self, begin: usize, end: usize) -> &str {
        &self.builder.source[begin..end]
    }

    fn remaining(&self) -> &str {
        self.slice(self.pos, self.source().len())
    }

    fn advance(&mut self) -> Option<char> {
        match self.remaining().chars().next() {
            Some(c) => {
                self.pos += c.len_utf8();
                Some(c)
            }

            None => None,
        }
    }

    fn consume(&mut self, expected: char) -> anyhow::Result<()> {
        if self.peek() != Some(expected) {
            bail!(
                "Expected '{}', got '{}'",
                expected,
                self.peek().unwrap_or('?')
            );
        }

        self.advance();
        Ok(())
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn skip_ascii_whitespace(&mut self) {
        while self
            .remaining()
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.pos += 1;
        }
    }

    fn consume_while(&mut self, mut f: impl FnMut(char) -> bool) {
        while let Some(c) = self.peek() {
            if !f(c) {
                break;
            }

            self.advance();
        }
    }

    fn current_parent(&self) -> NodeId {
        self.builder.current_parent()
    }

    fn push_tag(&mut self, tag: ParsingTag) -> anyhow::Result<()> {
        self.builder.push_tag(tag)
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

    fn push_tag(&mut self, tag: ParsingTag) -> anyhow::Result<()> {
        match tag.kind {
            TagKind::Open => {
                let id = self.push_tag_node(self.current_parent(), tag.tag, tag.attributes);
                self.unfinished.push((tag.tag, id));
            }

            TagKind::Close => match self.unfinished.last() {
                Some((last, _)) => {
                    if *last != tag.tag {
                        bail!("mismatched tag: expected {:?}, got {:?}", last, tag.tag);
                    }

                    self.unfinished.pop();
                }

                None => bail!("unmatched close tag: {:?}", tag.tag),
            },

            TagKind::SelfClosing => {
                self.push_tag_node(self.current_parent(), tag.tag, tag.attributes);
            }
        };

        Ok(())
    }

    fn push_tag_node(&mut self, parent: NodeId, tag: Tag, attributes: Vec<Attribute>) -> NodeId {
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

        self.arena[parent.0]
            .kind
            .children_mut()
            .expect("parser should not append children to text nodes")
            .push(id);

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

pub fn parse(source: String) -> anyhow::Result<Document> {
    let mut parser = Parser::new(source);

    while let Some(c) = parser.peek() {
        if c == '<' {
            let tag = parse_tag(&mut parser)?;
            parser.push_tag(tag)?;
        } else {
            let text = parse_text(&mut parser);
            parser.push_text(text);
        }
    }

    Ok(parser.finish())
}

fn parse_attributes(parser: &mut Parser) -> anyhow::Result<Vec<Attribute>> {
    let mut attributes = Vec::new();

    loop {
        parser.skip_ascii_whitespace();

        if matches!(parser.peek(), None | Some('>' | '/')) {
            break;
        }

        attributes.push(parse_attribute(parser)?);
    }

    Ok(attributes)
}

fn parse_attribute(parser: &mut Parser) -> anyhow::Result<Attribute> {
    let name_start = parser.pos;
    parser.consume_while(|c| !c.is_ascii_whitespace() && c != '=' && c != '>' && c != '/');

    if parser.pos == name_start {
        bail!("missing attribute name");
    }

    let name = name_start..parser.pos;
    parser.skip_ascii_whitespace();

    let value = if parser.match_char('=') {
        parser.skip_ascii_whitespace();
        Some(parse_attribute_value(parser)?)
    } else {
        None
    };

    Ok(Attribute { name, value })
}

fn parse_attribute_value(parser: &mut Parser) -> anyhow::Result<Range<usize>> {
    match parser.peek() {
        Some('"') | Some('\'') => {
            let Some(quote) = parser.advance() else {
                bail!("missing attribute value");
            };
            let start = parser.pos;

            while let Some(c) = parser.peek() {
                if c == quote {
                    let value = start..parser.pos;
                    parser.advance();
                    return Ok(value);
                }

                parser.advance();
            }

            bail!("attribute value is not closed")
        }

        Some(_) => {
            let start = parser.pos;
            parser.consume_while(|c| !c.is_ascii_whitespace() && c != '>');
            Ok(start..parser.pos)
        }

        None => bail!("missing attribute value"),
    }
}

fn parse_tag(parser: &mut Parser) -> anyhow::Result<ParsingTag> {
    parser.consume('<')?;
    parser.skip_ascii_whitespace();

    let closing = parser.match_char('/');
    parser.skip_ascii_whitespace();

    let tag_start = parser.pos;
    parser.consume_while(|c| !c.is_ascii_whitespace() && c != '/' && c != '>');

    if parser.pos == tag_start {
        bail!("missing tag name");
    }

    let name = parser.slice(tag_start, parser.pos);
    let tag = Tag::parse(name);
    let is_markup_declaration = name.starts_with('!');
    let attributes = if closing {
        Vec::new()
    } else {
        parse_attributes(parser)?
    };

    parser.skip_ascii_whitespace();
    let explicit_self_closing = parser.match_char('/');
    parser.skip_ascii_whitespace();
    parser.consume('>')?;

    let parsing_tag = if closing {
        ParsingTag {
            tag,
            attributes,
            kind: TagKind::Close,
        }
    } else if explicit_self_closing || tag.is_self_closing() || is_markup_declaration {
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
    };

    Ok(parsing_tag)
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
    attributes: Vec<Attribute>,
    kind: TagKind,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_text_under_document_root() {
        let document = parse("hello".to_string()).unwrap();
        let text = document.children(document.root())[0];

        assert_eq!(document.text(text), Some("hello"));
        assert_eq!(document.parent(text), Some(document.root()));
    }

    #[test]
    fn parses_nested_tags() {
        let document = parse("<p>Hello <em>world</em></p>".to_string()).unwrap();
        let paragraph = document.children(document.root())[0];
        let emphasis = document.children(paragraph)[1];
        let text = document.children(emphasis)[0];

        assert_eq!(document.tag(paragraph), Some(Tag::P));
        assert_eq!(document.tag(emphasis), Some(Tag::Em));
        assert_eq!(document.text(text), Some("world"));
    }

    #[test]
    fn parses_attributes() {
        let document =
            parse("<a href=http://example.org class=\"external\" disabled>link</a>".to_string())
                .unwrap();
        let link = document.children(document.root())[0];

        assert_eq!(document.tag(link), Some(Tag::A));
        let attributes = match document.view(link) {
            crate::html::NodeView::Tag { attributes, .. } => attributes,
            _ => &[],
        };

        assert_eq!(attributes.len(), 3);
        assert_eq!(attributes[0].name(&document), "href");
        assert_eq!(attributes[0].value(&document), Some("http://example.org"));
        assert_eq!(attributes[1].name(&document), "class");
        assert_eq!(attributes[1].value(&document), Some("external"));
        assert_eq!(attributes[2].name(&document), "disabled");
        assert_eq!(attributes[2].value(&document), None);
    }

    #[test]
    fn parses_quoted_attribute_values_with_spaces_and_greater_than() {
        let document = parse(
            "<a title=\"hello > world with spaces\" href='http://example.org?q=a>b'>link</a>"
                .to_string(),
        )
        .unwrap();
        let link = document.children(document.root())[0];
        let attributes = match document.view(link) {
            crate::html::NodeView::Tag { attributes, .. } => attributes,
            _ => &[],
        };

        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes[0].name(&document), "title");
        assert_eq!(
            attributes[0].value(&document),
            Some("hello > world with spaces")
        );
        assert_eq!(attributes[1].name(&document), "href");
        assert_eq!(
            attributes[1].value(&document),
            Some("http://example.org?q=a>b")
        );
    }

    #[test]
    fn greater_than_inside_quoted_attribute_does_not_close_tag() {
        let document = parse("<p data-x=\"1 > 0\">ok</p>".to_string()).unwrap();
        let paragraph = document.children(document.root())[0];
        let text = document.children(paragraph)[0];
        let attributes = match document.view(paragraph) {
            crate::html::NodeView::Tag { attributes, .. } => attributes,
            _ => &[],
        };

        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].name(&document), "data-x");
        assert_eq!(attributes[0].value(&document), Some("1 > 0"));
        assert_eq!(document.text(text), Some("ok"));
    }

    #[test]
    fn treats_void_tags_as_self_closing() {
        let document = parse("<p>a<br>b</p>".to_string()).unwrap();
        let paragraph = document.children(document.root())[0];
        let children = document.children(paragraph);

        assert_eq!(document.text(children[0]), Some("a"));
        assert_eq!(document.tag(children[1]), Some(Tag::Br));
        assert_eq!(document.text(children[2]), Some("b"));
    }

    #[test]
    fn auto_closes_unfinished_tags_at_eof() {
        let document = parse("<p>hello".to_string()).unwrap();
        let paragraph = document.children(document.root())[0];
        let text = document.children(paragraph)[0];

        assert_eq!(document.tag(paragraph), Some(Tag::P));
        assert_eq!(document.text(text), Some("hello"));
    }
}
