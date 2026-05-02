#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Html,
    Head,
    Title,
    Style,
    Script,
    Meta,
    Link,
    Body,

    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    P,
    Div,
    Span,
    Br,
    Hr,
    Img,
    Input,
    Strong,
    Em,
    B,
    I,
    Small,
    Big,
    Header,
    Nav,
    Main,
    Section,
    Article,
    Aside,
    Footer,
    Ul,
    Ol,
    Li,
    A,

    Unknown,
}

impl Tag {
    pub fn parse(raw: &str) -> Self {
        match raw {
            "html" => Tag::Html,
            "head" => Tag::Head,
            "title" => Tag::Title,
            "style" => Tag::Style,
            "script" => Tag::Script,
            "meta" => Tag::Meta,
            "link" => Tag::Link,
            "body" => Tag::Body,
            "h1" => Tag::H1,
            "h2" => Tag::H2,
            "h3" => Tag::H3,
            "h4" => Tag::H4,
            "h5" => Tag::H5,
            "h6" => Tag::H6,
            "p" => Tag::P,
            "div" => Tag::Div,
            "span" => Tag::Span,
            "br" => Tag::Br,
            "hr" => Tag::Hr,
            "img" => Tag::Img,
            "input" => Tag::Input,
            "strong" => Tag::Strong,
            "em" => Tag::Em,
            "b" => Tag::B,
            "i" => Tag::I,
            "small" => Tag::Small,
            "big" => Tag::Big,
            "header" => Tag::Header,
            "nav" => Tag::Nav,
            "main" => Tag::Main,
            "section" => Tag::Section,
            "article" => Tag::Article,
            "aside" => Tag::Aside,
            "footer" => Tag::Footer,
            "ul" => Tag::Ul,
            "ol" => Tag::Ol,
            "li" => Tag::Li,
            "a" => Tag::A,
            _unknown => Tag::Unknown,
        }
    }

    pub fn is_self_closing(self) -> bool {
        matches!(
            self,
            Tag::Br | Tag::Hr | Tag::Img | Tag::Input | Tag::Link | Tag::Meta
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagKind {
    Open,
    Close,
    SelfClosing,
}
