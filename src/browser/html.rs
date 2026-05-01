#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Text(String),
    Tag(String),
}

pub fn lex(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut buffer = String::new();
    let mut in_tag = false;

    for c in source.chars() {
        if c == '<' && !in_tag {
            if !buffer.is_empty() {
                tokens.push(Token::Text(std::mem::take(&mut buffer)));
            }
            in_tag = true;
        } else if c == '>' && in_tag {
            tokens.push(Token::Tag(std::mem::take(&mut buffer)));
            in_tag = false;
        } else {
            buffer.push(c);
        }
    }

    if !buffer.is_empty() {
        if in_tag {
            buffer.insert(0, '<');
        }
        tokens.push(Token::Text(buffer));
    }

    tokens
}
