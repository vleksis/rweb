use crate::html::Tag;
use crate::html::document::Attribute;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Token {
    Comment(String),
    Doctype(String),
    Text(String),
    OpenTag {
        tag: Tag,
        attributes: Vec<Attribute>,
    },
    SelfClosingTag {
        tag: Tag,
        attributes: Vec<Attribute>,
    },
    CloseTag(Tag),
    Eof,
}
