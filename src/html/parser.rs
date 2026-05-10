use anyhow::bail;

use crate::html::document::Attribute;
use crate::html::document::Document;
use crate::html::document::DocumentNode;
use crate::html::document::DomId;
use crate::html::document::DomKind;
use crate::html::document::DomNode;
use crate::html::document::TagNode;
use crate::html::document::TextNode;
use crate::html::lexer::Lexer;
use crate::html::tag::Tag;
use crate::html::tag::TagKind;
use crate::html::token::Token;

#[derive(Debug)]
struct DocumentBuilder {
    arena: Vec<DomNode>,
    unfinished: Vec<(Tag, DomId)>,
}

impl DocumentBuilder {
    fn new() -> Self {
        let root = DomNode {
            parent: None,
            kind: DomKind::Document(DocumentNode {
                children: Vec::new(),
            }),
        };

        Self {
            arena: vec![root],
            unfinished: Vec::new(),
        }
    }

    fn current_parent(&self) -> DomId {
        self.unfinished
            .last()
            .map(|(_, id)| *id)
            .unwrap_or_else(|| DomId(0))
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

    fn push_tag_node(&mut self, parent: DomId, tag: Tag, attributes: Vec<Attribute>) -> DomId {
        self.push_node(
            parent,
            DomKind::Tag(TagNode {
                tag,
                attributes,
                children: Vec::new(),
            }),
        )
    }

    fn push_text(&mut self, parent: DomId, text: String) -> DomId {
        self.push_node(parent, DomKind::Text(TextNode { text }))
    }

    fn push_node(&mut self, parent: DomId, kind: DomKind) -> DomId {
        let id = DomId(self.arena.len());

        self.arena[parent.0]
            .kind
            .children_mut()
            .expect("parser should not append children to text nodes")
            .push(id);

        self.arena.push(DomNode {
            parent: Some(parent),
            kind,
        });
        id
    }

    fn build(self) -> Vec<DomNode> {
        self.arena
    }
}

pub fn parse(source: String) -> anyhow::Result<Document> {
    let arena = parse_nodes(&source)?;

    Ok(Document::from_parts(source, arena))
}

fn parse_nodes(source: &str) -> anyhow::Result<Vec<DomNode>> {
    let mut builder = DocumentBuilder::new();
    let mut lexer = Lexer::new(source);

    loop {
        match lexer.next()? {
            Token::Comment(_) | Token::Doctype(_) => {}
            Token::Text(text) => {
                let parent = builder.current_parent();
                builder.push_text(parent, text);
            }
            Token::OpenTag { tag, attributes } => {
                builder.push_tag(ParsingTag {
                    tag,
                    attributes,
                    kind: TagKind::Open,
                })?;
            }
            Token::SelfClosingTag { tag, attributes } => {
                builder.push_tag(ParsingTag {
                    tag,
                    attributes,
                    kind: TagKind::SelfClosing,
                })?;
            }
            Token::CloseTag(tag) => {
                builder.push_tag(ParsingTag {
                    tag,
                    attributes: Vec::new(),
                    kind: TagKind::Close,
                })?;
            }
            Token::Eof => break,
        }
    }

    Ok(builder.build())
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
        assert_eq!(attributes[0].name(), "href");
        assert_eq!(attributes[0].value(), Some("http://example.org"));
        assert_eq!(attributes[1].name(), "class");
        assert_eq!(attributes[1].value(), Some("external"));
        assert_eq!(attributes[2].name(), "disabled");
        assert_eq!(attributes[2].value(), None);
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
        assert_eq!(attributes[0].name(), "title");
        assert_eq!(attributes[0].value(), Some("hello > world with spaces"));
        assert_eq!(attributes[1].name(), "href");
        assert_eq!(attributes[1].value(), Some("http://example.org?q=a>b"));
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
        assert_eq!(attributes[0].name(), "data-x");
        assert_eq!(attributes[0].value(), Some("1 > 0"));
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
