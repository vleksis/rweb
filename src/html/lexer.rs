use anyhow::Context;
use anyhow::bail;

use crate::html::Tag;
use crate::html::document::Attribute;
use crate::html::token::Token;

#[derive(Debug)]
pub(super) struct Lexer<'s> {
    source: &'s str,
    pos: usize,
}

impl<'s> Lexer<'s> {
    pub(super) fn new(source: &'s str) -> Self {
        Self { source, pos: 0 }
    }

    pub(super) fn next(&mut self) -> anyhow::Result<Token> {
        scan_token(self)
    }

    fn peek(&self) -> Option<char> {
        self.nth(0)
    }

    fn nth(&self, n: usize) -> Option<char> {
        self.remaining().chars().nth(n)
    }

    fn slice(&self, begin: usize, end: usize) -> &'s str {
        &self.source[begin..end]
    }

    fn slice_from(&self, begin: usize) -> &'s str {
        self.slice(begin, self.pos)
    }

    fn remaining(&self) -> &'s str {
        self.slice(self.pos, self.source.len())
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
        if self.consume_if(expected) {
            return Ok(());
        }

        match self.peek() {
            Some(c) => bail!("Expected '{}', got '{}'", expected, c),
            None => {
                bail!("Expected '{}', got EOF", expected,)
            }
        }
    }

    fn consume_str(&mut self, expected: &str) -> anyhow::Result<()> {
        if self.consume_str_if(expected) {
            Ok(())
        } else {
            bail!("expected {expected}");
        }
    }

    fn consume_until(&mut self, mut pred: impl FnMut(char) -> bool) {
        while let Some(c) = self.peek() {
            if pred(c) {
                break;
            }

            self.advance();
        }
    }

    fn skip_whitespaces(&mut self) {
        while self.peek().as_ref().is_some_and(char::is_ascii_whitespace) {
            self.advance();
        }
    }

    fn match_char(&self, expected: char) -> bool {
        self.peek() == Some(expected)
    }

    fn match_str(&self, expected: &str) -> bool {
        self.remaining().starts_with(expected)
    }

    fn consume_if(&mut self, expected: char) -> bool {
        if !self.match_char(expected) {
            return false;
        }

        self.advance();
        true
    }

    fn consume_str_if(&mut self, expected: &str) -> bool {
        if !self.match_str(expected) {
            return false;
        }

        self.pos += expected.len();
        true
    }

    fn is_eof(&self) -> bool {
        self.peek().is_none()
    }
}

pub(super) fn scan_token(lexer: &mut Lexer) -> anyhow::Result<Token> {
    let Some(cur) = lexer.peek() else {
        return Ok(Token::Eof);
    };

    if lexer.match_str("<!--") {
        return scan_comment(lexer);
    }

    if cur == '<' {
        return scan_tag(lexer);
    }

    Ok(scan_text(lexer))
}

fn scan_comment(lexer: &mut Lexer) -> anyhow::Result<Token> {
    lexer.consume_str("<!--")?;
    let start = lexer.pos;

    while !lexer.match_str("-->") {
        if lexer.is_eof() {
            bail!("comment is not closed");
        }

        lexer.advance();
    }

    let comment = lexer.slice_from(start).to_owned();
    lexer.consume_str("-->")?;

    Ok(Token::Comment(comment))
}

fn scan_tag(lexer: &mut Lexer) -> anyhow::Result<Token> {
    lexer.consume('<')?;
    lexer.skip_whitespaces();

    if lexer.consume_if('!') {
        return scan_doctype(lexer);
    }

    if lexer.consume_if('/') {
        lexer.skip_whitespaces();

        let tag = scan_tag_name(lexer)?;
        lexer.skip_whitespaces();
        lexer.consume('>')?;

        return Ok(Token::CloseTag(tag));
    }

    let tag = scan_tag_name(lexer)?;
    let attributes = scan_attributes(lexer)?;

    lexer.skip_whitespaces();
    let _explicit_self_closing = lexer.consume_if('/');
    lexer.skip_whitespaces();
    lexer.consume('>')?;

    let token = if tag.is_self_closing() {
        Token::SelfClosingTag { tag, attributes }
    } else {
        Token::OpenTag { tag, attributes }
    };

    Ok(token)
}

fn scan_doctype(lexer: &mut Lexer) -> anyhow::Result<Token> {
    let start = lexer.pos;
    lexer.consume_until(|c| c == '>');
    let doctype = lexer.slice_from(start).to_owned();
    lexer.consume('>')?;

    Ok(Token::Doctype(doctype))
}

fn scan_attributes(lexer: &mut Lexer) -> anyhow::Result<Vec<Attribute>> {
    let mut attributes = Vec::new();

    loop {
        lexer.skip_whitespaces();
        if matches!(lexer.peek(), None | Some('>') | Some('/')) {
            break;
        }

        let attr = scan_attribute(lexer)?;
        attributes.push(attr)
    }

    Ok(attributes)
}

fn scan_attribute(lexer: &mut Lexer) -> anyhow::Result<Attribute> {
    let start = lexer.pos;

    lexer.consume_until(|c| c.is_ascii_whitespace() || c == '=' || c == '>' || c == '/');

    if lexer.pos == start {
        bail!("missing attribute name");
    }

    let name = lexer.slice_from(start).to_owned();

    lexer.skip_whitespaces();
    let value = if lexer.consume_if('=') {
        lexer.skip_whitespaces();
        Some(scan_attribute_value(lexer)?)
    } else {
        None
    };

    Ok(Attribute { name, value })
}

fn scan_attribute_value(lexer: &mut Lexer) -> anyhow::Result<String> {
    match lexer.peek() {
        Some('"') | Some('\'') => {
            let value = scan_quoted_string(lexer)?;
            Ok(value)
        }

        Some(_) => {
            let start = lexer.pos;
            lexer.consume_until(|c| c.is_ascii_whitespace() || c == '>');
            Ok(lexer.slice_from(start).to_owned())
        }

        None => bail!("missing attribute value"),
    }
}

fn scan_quoted_string(lexer: &mut Lexer) -> anyhow::Result<String> {
    let quote = lexer.advance().context("peek returned quote")?;
    let start = lexer.pos;

    while lexer.peek() != Some(quote) {
        if lexer.is_eof() {
            bail!("attribute value is not closed");
        }

        lexer.advance();
    }

    let value = lexer.slice_from(start).to_owned();
    lexer.consume(quote)?;

    Ok(value)
}

fn scan_tag_name(lexer: &mut Lexer) -> anyhow::Result<Tag> {
    let start = lexer.pos;

    lexer.consume_until(|c| c.is_ascii_whitespace() || c == '/' || c == '>');
    if lexer.pos == start {
        bail!("missing tag name");
    }

    Ok(Tag::parse(lexer.slice_from(start)))
}

fn scan_text(lexer: &mut Lexer) -> Token {
    let start = lexer.pos;
    lexer.consume_until(|c| c == '<');
    let text = lexer.slice_from(start).to_owned();
    Token::Text(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_one(source: &str) -> anyhow::Result<Token> {
        Lexer::new(source).next()
    }

    #[test]
    fn lexes_text() {
        assert_eq!(lex_one("hello").unwrap(), Token::Text("hello".to_string()));
    }

    #[test]
    fn preserves_text_whitespace() {
        assert_eq!(
            lex_one(" hello world ").unwrap(),
            Token::Text(" hello world ".to_string())
        );
    }

    #[test]
    fn lexes_open_tag_with_attributes() {
        assert_eq!(
            lex_one("<a href=http://example.org class=\"external\" disabled>").unwrap(),
            Token::OpenTag {
                tag: Tag::A,
                attributes: vec![
                    Attribute {
                        name: "href".to_string(),
                        value: Some("http://example.org".to_string()),
                    },
                    Attribute {
                        name: "class".to_string(),
                        value: Some("external".to_string()),
                    },
                    Attribute {
                        name: "disabled".to_string(),
                        value: None,
                    },
                ],
            }
        );
    }

    #[test]
    fn lexes_close_tag() {
        assert_eq!(lex_one("</p>").unwrap(), Token::CloseTag(Tag::P));
    }

    #[test]
    fn lexes_void_tag_as_self_closing() {
        assert_eq!(
            lex_one("<br>").unwrap(),
            Token::SelfClosingTag {
                tag: Tag::Br,
                attributes: Vec::new(),
            }
        );
    }

    #[test]
    fn normalizes_non_void_slash_tag_to_open_tag() {
        assert_eq!(
            lex_one("<div />").unwrap(),
            Token::OpenTag {
                tag: Tag::Div,
                attributes: Vec::new(),
            }
        );
    }

    #[test]
    fn lexes_quoted_attribute_values_with_greater_than() {
        assert_eq!(
            lex_one("<p data-x=\"1 > 0\">").unwrap(),
            Token::OpenTag {
                tag: Tag::P,
                attributes: vec![Attribute {
                    name: "data-x".to_string(),
                    value: Some("1 > 0".to_string()),
                }],
            }
        );
    }

    #[test]
    fn lexes_comment() {
        assert_eq!(
            lex_one("<!-- hidden -->").unwrap(),
            Token::Comment(" hidden ".to_string())
        );
    }

    #[test]
    fn lexes_doctype() {
        assert_eq!(
            lex_one("<!doctype html>").unwrap(),
            Token::Doctype("doctype html".to_string())
        );
    }

    #[test]
    fn returns_eof_at_end() {
        let mut lexer = Lexer::new("hello");

        assert_eq!(lexer.next().unwrap(), Token::Text("hello".to_string()));
        assert_eq!(lexer.next().unwrap(), Token::Eof);
    }
}
