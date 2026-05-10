mod document;
mod lexer;
mod parser;
mod tag;
mod token;

pub use document::Document;
pub use document::DomId;
pub use document::NodeView;
pub use parser::parse;
pub use tag::Tag;
